use std::collections::HashSet;

use bit_vec::BitVec;
use visioncortex::{BoundingRect, Color, ColorImage, ColorName, PathI32, PointI32, color_clusters::{Runner, RunnerConfig}};
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
    pub fn new_from_canvas_id_and_mask(canvas_id: &str, display_selector: DisplaySelector, display_tangents: bool, display_control_points: bool, x: usize, y: usize, w: usize, h: usize) -> Self {
        let draw_util = DrawUtil::new(canvas_id, display_selector, display_tangents, display_control_points);
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
        let mut endpoints = HashSet::new();
        let path_segments: Vec<PathI32> = paths.into_iter()
                                               .map(|path| {
                                                   self.find_segments_on_path_with_unique_endpoints(path, &mut endpoints, simplify_tolerance)
                                               })
                                               .flatten()
                                               .collect();

        //# Matching paths
        let match_item_set = self.construct_match_item_set(&path_segments);
        let matchings = Matcher::find_all_possible_matchings(match_item_set);

        let correct_tail_tangents = false;
        if !Self::try_interpolate_with_matchings(&matchings, &path_segments, self.hole_rect, &self.draw_util, correct_tail_tangents) {
            let correct_tail_tangents = true;
            if !Self::try_interpolate_with_matchings(&matchings, &path_segments, self.hole_rect, &self.draw_util, correct_tail_tangents) {
                panic!("Still not drawn!");
            }
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
    fn find_segments_on_path_with_unique_endpoints(&self, path: PathI32, current_endpoints: &mut HashSet<PointI32>, simplify_tolerance: f64) -> Vec<PathI32> {
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

        endpoints_iter.filter_map(|endpoint| {
            let inserted = current_endpoints.insert(path[endpoint]);
            if inserted {
                Some(self.walk_segment(&path, endpoint, &is_boundary_mask, simplify_tolerance))
            } else {
                None
            }
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
    fn construct_match_item_set(&self, path_segments: &[PathI32]) -> MatchItemSet {
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
        match_item_set
    }

    /// Return true iff one of the matchings is successfully interpolated
    fn try_interpolate_with_matchings(matchings: &[Matching], path_segments: &[PathI32], hole_rect: BoundingRect, draw_util: &DrawUtil, correct_tail_tangents: bool) -> bool {
        let mut drawn = false;

        'matching_loop: for matching in matchings.iter() {
            let mut interpolated_curves = vec![];
            for &(index1, index2) in matching.iter() {
                let (curve1, curve2) = (path_segments[index1].to_path_f64(), path_segments[index2].to_path_f64());

                if draw_util.display_selector == DisplaySelector::Simplified {
                    let color1 = Color::get_palette_color(1);
                    let color2 = Color::get_palette_color(3);
                    draw_util.draw_path_f64(&color1, &curve1);
                    draw_util.draw_path_f64(&color2, &curve2);
                }

                let curve_interpolator_config = CurveInterpolatorConfig::default();
                let curve_interpolator = CurveInterpolator::new(curve_interpolator_config, hole_rect, draw_util.clone());
                
                let control_points_retract_ratio = 1.0 / 1.618;
                if let Some(interpolated_curve) = curve_interpolator.interpolate_curve_between_curves(curve1, curve2, false, false, correct_tail_tangents, control_points_retract_ratio) {
                    interpolated_curves.push(interpolated_curve);
                } else {
                    // A curve cannot be interpolated, this matching is wrong
                    continue 'matching_loop;
                }
            }
            // If all curves can be interpolated without problems, draw them
            interpolated_curves.into_iter().for_each(|interpolated_curve| {
                draw_util.draw_compound_path(&Color::get_palette_color(4), &interpolated_curve);
            });
            drawn = true;
            // Trust it to be the correct solution
            break;
        }

        drawn
    }
}
