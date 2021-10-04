use bit_vec::BitVec;
use visioncortex::{BoundingRect, Color, ColorImage, ColorName, PathI32, PointI32, color_clusters::{Runner, RunnerConfig}};
use wasm_bindgen::prelude::*;

use crate::image_repair::{CurveInterpolator, CurveInterpolatorConfig};

use super::{draw::{DisplaySelector, DrawUtil}};

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

        // paths.iter().enumerate().for_each(|(i, path)| {
        //     console_log_util(format!("{:?}", path.path));
        //     self.draw_util.draw_path(Color::get_palette_color(i+1), path);
        // });

        //# Path identification and splitting
        let paths_with_two_endpoints: Vec<(PathI32, Vec<usize>)> = paths
            .into_iter()
            .filter_map(|path| {
                let endpoints = self.find_endpoints_on_path(&path);
                if endpoints.len() == 2 {
                    Some((path, endpoints))
                } else {
                    None
                }
            })
            .collect();

        // Assume only 1 path with 2 endpoints
        assert!(paths_with_two_endpoints.len() == 1);
        let (path, endpoints) = paths_with_two_endpoints[0].clone();

        let tail1 = endpoints[0];
        let tail2 = endpoints[1];

        let color1 = Color::get_palette_color(1);
        let color2 = Color::get_palette_color(3);

        let (curve1, curve2) = self.split_path(path, tail1, tail2);
        if self.draw_util.display_selector == DisplaySelector::Raw {
            self.draw_util.draw_path_i32(&color1, &curve1);
            self.draw_util.draw_path_i32(&color2, &curve2);
        }

        let curve_interpolator_config = CurveInterpolatorConfig::default();
        let curve_interpolator = CurveInterpolator::new(curve_interpolator_config, self.hole_rect, self.draw_util.clone());

        let interpolated_curve = curve_interpolator.interpolate_curve_between_curves(curve1.to_path_f64(), curve2.to_path_f64(), true, true, &self.draw_util);
        
        self.draw_util.draw_compound_path(&Color::get_palette_color(4), &interpolated_curve);
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

    /// Currently naive approach of checking whether points in the path are on the boundary of hole_rect.
    /// Return indices of points on 'path' that are on said boundary.
    /// Note that there are a cluster of points on the boundary, but only the 2 ends of the cluster are
    /// returned.
    fn find_endpoints_on_path(&self, path: &PathI32) -> Vec<usize> {
        let path = path.to_unclosed();
        let len = path.len();
        let is_boundary_mask = BitVec::from_fn(len, |i| {
            self.hole_rect.have_point_on_boundary(path[i])
        });

        let num_boundary_points = is_boundary_mask.iter().filter(|b| *b).count();

        if num_boundary_points == 0 {
            return vec![];
        }

        if num_boundary_points == len {
            return vec![0, len-1];
        }

        if !(is_boundary_mask[0] && is_boundary_mask[len-1]) {
            let boundary_idx: Vec<usize> = is_boundary_mask.into_iter()
                            .enumerate()
                            .filter_map(|(i, b)| {
                                if b {
                                    Some(i)
                                } else {
                                    None
                                }
                            })
                            .collect();
            let min = *boundary_idx.iter().min().unwrap();
            let max = boundary_idx.into_iter().max().unwrap();
            return vec![min, max];
        }

        // Find the endpoint starting from 0
        let mut endpoint1 = 0;
        while is_boundary_mask[endpoint1] { endpoint1 += 1; }
        endpoint1 -= 1;

        // Find the endpoint starting from len-1
        let mut endpoint2 = len-1;
        while is_boundary_mask[endpoint2] { endpoint2 -= 1; }
        endpoint2 += 1;

        vec![endpoint1, endpoint2]
    }

    /// Starting from a determined midpoint that is not on the boundary of 'hole_rect',
    /// divide 'path' into two subpaths ending at 'tail1' and 'tail2'.
    /// The behavior is also undefined unless tail1 != tail2 < path.len(), assuming 'path' is in an unclosed form.
    /// The behavior is also undefined unless both 'tail1' and 'tail2' correspond to points on the boundary of 'hole_rect'.
    /// The behavior is also undefined unless both 'tail1' and 'tail2' have exactly 1 neighbor that is not on the boundary
    /// of 'hole_rect'.
    fn split_path(&self, path: PathI32, tail1: usize, tail2: usize) -> (PathI32, PathI32) {
        let path = path.to_unclosed();
        let len = path.len();

        assert!(tail1 != tail2);
        assert!(tail1 < len && tail2 < len);

        assert!(self.hole_rect.have_point_on_boundary(path[tail1]) &&
                self.hole_rect.have_point_on_boundary(path[tail2]));

        // Find the (wrapped) neighbors of 'tail1' and 'tail2' that are not on the boundary of 'hole_rect'
        let neighbor_not_on_bound = |tail: usize| {
            let tail_neighbors_idx_dir = vec![(if tail == 0 {len-1} else {tail-1}, -1), ((tail+1) % len, 1)];
            let tail_neighbors_not_on_bound: Vec<(usize, i32)> = 
                tail_neighbors_idx_dir.into_iter()
                                .filter(|(idx, _dir)| {
                                    !self.hole_rect.have_point_on_boundary(path[*idx])
                                })
                                .collect();
            assert_eq!(1, tail_neighbors_not_on_bound.len());
            tail_neighbors_not_on_bound[0]
        };
        let (tail1_neighbor, tail1_direction) = neighbor_not_on_bound(tail1);
        let (tail2_neighbor, tail2_direction) = neighbor_not_on_bound(tail2);

        // Approach the midpoint from both tails, pushing the points into result subpaths along the way
        let mut tail1_approacher = tail1_neighbor as i32;
        let mut tail2_approacher = tail2_neighbor as i32;
        let mut tail1_points = vec![path[tail1], path[tail1_neighbor]];
        let mut tail2_points = vec![path[tail2], path[tail2_neighbor]];
        while tail1_approacher != tail2_approacher {
            // Approach from tail1
            tail1_approacher += tail1_direction;
            tail1_approacher = if tail1_approacher >= 0 {tail1_approacher % len as i32} else {len as i32 - 1};

            tail1_points.push(path[tail1_approacher as usize]);

            // Check for match
            if tail1_approacher == tail2_approacher {
                break;
            }

            // Approach from tail2
            tail2_approacher += tail2_direction;
            tail2_approacher = if tail2_approacher >= 0 {tail2_approacher % len as i32} else {len as i32 - 1};
            
            tail2_points.push(path[tail2_approacher as usize]);

            // Sanity check
            assert_ne!(tail1_approacher, tail1 as i32);
            assert_ne!(tail2_approacher, tail2 as i32);
        }

        // Want the two subpaths to point into 'hole_rect'
        tail1_points.reverse();
        tail2_points.reverse();

        (PathI32::from_points(tail1_points), PathI32::from_points(tail2_points))
    }
}
