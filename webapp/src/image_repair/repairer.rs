use bit_vec::BitVec;
use visioncortex::{BoundingRect, Color, ColorImage, ColorName, PathF64, PathI32, PointI32, color_clusters::{Runner, RunnerConfig}};
use wasm_bindgen::prelude::*;

use crate::{image_repair::{CurveInterpolator, CurveInterpolatorConfig, MatchItem, MatchItemSet, Matcher}, util::console_log_util};

use super::{Matching, draw::{DisplaySelector, DrawUtil}};

#[wasm_bindgen]
pub struct Repairer {
    image: ColorImage,
    hole_rect: BoundingRect,
    draw_util: DrawUtil,
}

// WASM API
#[wasm_bindgen]
impl Repairer {
    #[wasm_bindgen(constructor)]
    pub fn new_from_canvas_id_and_mask(canvas_id: &str, display_selector: DisplaySelector, display_tangents: bool, x: usize, y: usize, w: usize, h: usize) -> Self {
        let draw_util = DrawUtil::new(canvas_id, display_selector, display_tangents);
        let canvas = &draw_util.canvas;

        // Raw image
        let mut image = canvas.get_image_data_as_color_image(0, 0, canvas.width() as u32, canvas.height() as u32);

        let hole_rect = BoundingRect::new_x_y_w_h(x as i32, y as i32, w as i32, h as i32);

        let empty_color = Color::color(&ColorName::White);

        // Remove hole from image
        for x_offset in 0..hole_rect.width() {
            for y_offset in 0..hole_rect.height() {
                image.set_pixel(x + x_offset as usize, y + y_offset as usize, &empty_color)
            }
        }

        // Draw hole on canvas
        draw_util.fill_rect(&empty_color, x, y, w, h);

        Self { image, hole_rect, draw_util }
    }

    pub fn repair(&self) {
        //# Path walking
        let paths = self.get_paths();

        //# Path identification, segmentation, and simplification
        let simplify_tolerance = 2.0;
        let path_segments: Vec<PathI32> = paths.into_iter()
                                 .map(|path| {
                                    self.find_segments_on_path(path, simplify_tolerance)
                                 })
                                 .flatten()
                                 .collect();

        //# Matching paths
        let direction_difference_threshold = 0.15;
        let matching = self.find_matching(&path_segments, direction_difference_threshold);

        for (index1, index2) in matching.into_iter() {
            let (curve1, curve2) = (path_segments[index1].to_path_f64(), path_segments[index2].to_path_f64());

            if self.draw_util.display_selector == DisplaySelector::Simplified {
                let color1 = Color::get_palette_color(1);
                let color2 = Color::get_palette_color(3);
                self.draw_util.draw_path_f64(&color1, &curve1);
                self.draw_util.draw_path_f64(&color2, &curve2);
            }

            let curve_interpolator_config = CurveInterpolatorConfig::default();
            let curve_interpolator = CurveInterpolator::new(curve_interpolator_config, self.hole_rect, self.draw_util.clone());

            let interpolated_curve = curve_interpolator.interpolate_curve_between_curves(curve1, curve2, false, false);
            
            self.draw_util.draw_compound_path(&Color::get_palette_color(4), &interpolated_curve);
        }
    }
}

// Test-specific helper functions
impl Repairer {
    fn get_paths(&self) -> Vec<PathI32> {
        // Clustering
        let runner_config = RunnerConfig {
            diagonal: false,
            hierarchical: 64,
            batch_size: 25600,
            good_min_area: 1,
            good_max_area: self.image.width * self.image.height,
            is_same_color_a: 4,
            is_same_color_b: 1,
            deepen_diff: 0,
            hollow_neighbours: 1,
            key_color: Color::color(&ColorName::White),
        };
        let runner = Runner::new(runner_config, self.image.clone());
        let clusters = runner.run();
        let clusters_view = clusters.view();

        // Path walking
        clusters_view.clusters.iter().filter_map(|cluster| {
            let image = cluster.to_image(&clusters_view);
            let mut path = PathI32::image_to_path(&image, true, visioncortex::PathSimplifyMode::None);

            if path.len() <= 5 {
                None
            } else {
                // Apply offset to get coords in original image
                let offset = PointI32::new(cluster.rect.left, cluster.rect.top);
                path.offset(&offset);
                Some(path)
            }
        }).collect()
    }

    /// Return a vector of *simplified* path segments whose heads are endpoints, pointing outwards from hole_rect.
    /// Segments are walked until 'max_num_points' is reached or another boundary point is reached, whichever happens first.
    fn find_segments_on_path(&self, path: PathI32, simplify_tolerance: f64) -> Vec<PathI32> {
        let path = path.to_open();
        let len = path.len();
        let is_boundary_mask = BitVec::from_fn(len, |i| {
            self.hole_rect.have_point_on_boundary(path[i])
        });

        let endpoints_iter = (0..len).into_iter().filter(|&i| {
            let prev = if i == 0 {len-1} else {i-1};
            let next = (i + 1) % len;

            is_boundary_mask[i] && // itself is on boundary
            !(is_boundary_mask[prev] && is_boundary_mask[next]) // not both neighbors are on boundary
        });

        endpoints_iter.map(|endpoint| {
            self.walk_segment(&path, endpoint, &is_boundary_mask, simplify_tolerance)
        }).collect()
    }

    /// The behavior is undefined unless path.len() == is_boundary_mask.len().
    fn walk_segment(&self, path: &PathI32, endpoint_index: usize, is_boundary_mask: &BitVec<u32>, simplify_tolerance: f64) -> PathI32 {
        assert_eq!(path.len(), is_boundary_mask.len());

        // Determine direction
        let len = path.len();
        let prev = if endpoint_index == 0 {len-1} else {endpoint_index-1};
        let next = (endpoint_index + 1) % len;
        assert!(is_boundary_mask[prev] != is_boundary_mask[next]);
        let direction = if is_boundary_mask[prev] {1} else {-1};

        // Walk from 'endpoint_index' along 'path' by 'direction'
        // until 'max_num_points' points are in the walked path, or another boundary point is added
        let mut path_segment = PathI32::new();
        let mut endpoint = endpoint_index as i32;
        let len = len as i32;
        loop {
            path_segment.add(path[endpoint as usize]);

            endpoint += direction;
            endpoint = if endpoint >= 0 {endpoint % len} else {len-1};

            if is_boundary_mask[endpoint as usize] {
                path_segment.add(path[endpoint as usize]);
                break;
            }
        }

        // Simplify 'path_segment'
        PathI32::from_points(visioncortex::reduce::reduce(&path_segment.path, simplify_tolerance))
    }

    /// The behavior is undefined unless 'path_segments' has an even number of elements.
    /// The behavior is also undefined unless every segment has at least 2 points.
    /// The behavior is also undefined unless all segments have their tails at index 0.
    fn find_matching(&self, path_segments: &[PathI32], direction_difference_threshold: f64) -> Matching {
        assert_eq!(path_segments.len() % 2, 0);
        let match_items_iter = path_segments.iter()
                                            .map(|segment| {
                                                assert!(segment.len() >= 2);
                                                // 0 is tail
                                                let direction = (segment[0] - segment[1]).to_point_f64().get_normalized();
                                                MatchItem::new_with_default_id(segment[0].to_point_f64(), direction)
                                            });
        let mut match_item_set = MatchItemSet::new();
        match_items_iter.for_each(|match_item| {match_item_set.push_and_set_id(match_item)});
        Matcher::find_matching(match_item_set, direction_difference_threshold)
    }
}
