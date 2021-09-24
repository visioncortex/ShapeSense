use bit_vec::BitVec;
use flo_curves::{BezierCurve, BezierCurveFactory, bezier};
use visioncortex::{BoundingRect, Color, ColorImage, ColorName, PathF64, PathI32, PointF64, PointI32, color_clusters::{Runner, RunnerConfig}};
use wasm_bindgen::prelude::*;

use crate::{image_repair::{calculate_intersection, calculate_midpoint, find_corners, find_new_point_from_4_point_scheme}, util::console_log_util};

use super::draw::DrawUtil;

#[wasm_bindgen]
#[derive(PartialEq)]
pub enum DisplaySelector {
    None,
    Raw,
    Simplified,
    Smoothed,
}

#[wasm_bindgen]
pub struct Repairer {
    draw_util: DrawUtil,
    display_selector: DisplaySelector,
    display_tangents: bool,
    image: ColorImage,
    hole_rect: BoundingRect,
}

// WASM API
#[wasm_bindgen]
impl Repairer {
    #[wasm_bindgen(constructor)]
    pub fn new_from_canvas_id_and_mask(canvas_id: &str, display_selector: DisplaySelector, display_tangents: bool, x: usize, y: usize, w: usize, h: usize) -> Self {
        let draw_util = DrawUtil::new_from_canvas_id(canvas_id);
        let canvas = &draw_util.canvas;

        // Raw image
        let mut image = canvas.get_image_data_as_color_image(0, 0, canvas.width() as u32, canvas.height() as u32);

        let hole_rect = BoundingRect::new_x_y_w_h(x as i32, y as i32, w as i32, h as i32);

        let empty_color = Color::color(&ColorName::White);

        // Remove hole
        for x_offset in 0..hole_rect.width() {
            for y_offset in 0..hole_rect.height() {
                image.set_pixel(x + x_offset as usize, y + y_offset as usize, &empty_color)
            }
        }

        // Draw hole on canvas
        draw_util.fill_rect(&empty_color, x, y, w, h);

        Self { draw_util, display_selector, display_tangents, image, hole_rect }
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
        if self.display_selector == DisplaySelector::Raw {
            self.draw_util.draw_path_i32(&color1, &curve1);
            self.draw_util.draw_path_i32(&color2, &curve2);
        }

        let interpolated_curve = self.interpolate_curve_between_curves(curve1.to_path_f64(), curve2.to_path_f64(), true, true, &self.draw_util);
        
        self.draw_util.draw_path_f64(&Color::get_palette_color(4), &interpolated_curve);
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

// API
impl Repairer {
    /// Interpolate the imaginary curve between two existing curves.
    /// The endpoints of the interpolated curve are defined by 'at_tail_curve1' and 'at_tail_curve2'.
    /// If 'at_tail_curve1' is true, the last point of 'curve1' is used as one of the endpoints of the curve, otherwise the first
    /// point (head) of 'curve1' is used. The same goes for 'at_tail_curve2' and 'curve2'.
    pub fn interpolate_curve_between_curves(&self, mut curve1: PathF64, mut curve2: PathF64, at_tail_curve1: bool, at_tail_curve2: bool, draw_util: &DrawUtil) -> PathF64 {
        let color1 = Color::get_palette_color(1);
        let color2 = Color::get_palette_color(3);

        // The rest of the algorithm assumes at_tail = true
        if !at_tail_curve1 {
            curve1.path.reverse();
        }
        if !at_tail_curve2 {
            curve2.path.reverse();
        }
        let (curve1, curve2) = (curve1, curve2);

        let (endpoint1, endpoint2) = (curve1[curve1.len()-1], curve2[curve2.len()-1]);
        let base_length = endpoint1.distance_to(endpoint2);

        //# Curve simplification
        let tolerance = 1.5;
        let simplified_curve1 = PathF64::from_points(visioncortex::reduce::reduce(&curve1.path, tolerance));
        let simplified_curve2 = PathF64::from_points(visioncortex::reduce::reduce(&curve2.path, tolerance));

        if self.display_selector == DisplaySelector::Simplified {
            draw_util.draw_path_f64(&color1, &simplified_curve1);
            draw_util.draw_path_f64(&color2, &simplified_curve2);
        }

        //# Curve smoothing
        let outset_ratio = 8.0;
        let min_segment_length = 4.0;
        let max_iterations = 2;
        let corner_threshold = std::f64::consts::FRAC_PI_2;

        let (smooth_curve1, corners1) = Self::smoothen_open_curve_iterative(simplified_curve1, outset_ratio, min_segment_length, max_iterations, corner_threshold);
        let (smooth_curve2, corners2) = Self::smoothen_open_curve_iterative(simplified_curve2, outset_ratio, min_segment_length, max_iterations, corner_threshold);

        if self.display_selector == DisplaySelector::Smoothed {
            draw_util.draw_path_f64(&color1, &smooth_curve1);
            draw_util.draw_path_f64(&color2, &smooth_curve2);
        }

        //# Tail tangent approximation
        let tail_tangent_n = 8;
        let tail_weight_multiplier = 1.5;
        let (smooth_curve1_len, smooth_curve2_len) = (smooth_curve1.len(), smooth_curve2.len());
        let tail_tangent1 = Self::calculate_weighted_average_tangent_at_tail(smooth_curve1, &corners1, std::cmp::min(tail_tangent_n, smooth_curve1_len), base_length, tail_weight_multiplier);
        let tail_tangent2 = Self::calculate_weighted_average_tangent_at_tail(smooth_curve2, &corners2, std::cmp::min(tail_tangent_n, smooth_curve2_len), base_length, tail_weight_multiplier);

        if self.display_tangents {
            let tangent_visual_length = (self.hole_rect.width() + self.hole_rect.height()) as f64 / 3.5;
            draw_util.draw_line_f64(&color1, endpoint1, endpoint1 + tail_tangent1 * tangent_visual_length);
            draw_util.draw_line_f64(&color2, endpoint2, endpoint2 + tail_tangent2 * tangent_visual_length);
        }
        
        //# Curve interpolation
        let smoothness = 100;
        Self::calculate_cubic_curve(endpoint1, tail_tangent1, endpoint2, tail_tangent2, smoothness)
    }
}

// Helper functions
impl Repairer {
    /// Apply the 4-point scheme subdivision on 'path' in a convolutional manner iteratively, preserving corners.
    /// The corners of the smoothed path are returned as a bool mask.
    /// Segments (at any point during iteration) shorter than 'min_segment_length' are not further subdivided.
    /// If no subdivision is performed, the iterative process is terminated early.
    /// 'path' is returned as-is if path.len() < 4
    fn smoothen_open_curve_iterative(mut path: PathF64, outset_ratio: f64, min_segment_length: f64, max_iterations: usize, corner_threshold: f64) -> (PathF64, Vec<bool>) {
        let mut corners = find_corners(&path, corner_threshold);

        if path.len() < 4  {
            return (path, corners);
        }
        
        for _ in 0..max_iterations {
            let can_terminate_early = Self::smoothen_open_curve_step(&mut path, &mut corners, outset_ratio, min_segment_length);

            // Early termination
            if can_terminate_early {
                break;
            }
        }

        (path, corners)
    }

    /// Return true if no subdivision is done in this step.
    fn smoothen_open_curve_step(path: &mut PathF64, corners: &mut Vec<bool>, outset_ratio: f64, min_segment_length: f64) -> bool {
        let mut new_points = vec![path[0]];
        let mut new_corners = vec![corners[0]];

        // Duplicate the last point to make sure all segments except the first are subdivided
        path.add(path[path.len()-1]);

        // Apply 4-point scheme on 'path' in a convolutional manner
        for (i, points) in path.path.windows(4).enumerate() {
            new_points.push(points[1]);
            new_corners.push(corners[i+1]);

            // Do not smooth out corners
            if corners[i+1] || corners[i+2] {
                continue;
            }

            // Threshold on segment length of the segment to be broken down
            let checked_segment_length = points[1].distance_to(points[2]);
            if checked_segment_length >= min_segment_length {
                new_points.push(find_new_point_from_4_point_scheme(
                    &points[1], &points[2], &points[0], &points[3], outset_ratio));
                new_corners.push(false); // New point must be a non-corner during subdivision
            }    
        }

        // Push the original last point
        new_points.extend(path.iter().rev().take(1));
        new_corners.push(corners[corners.len()-1]);

        assert_eq!(new_points.len(), new_corners.len());

        if new_points.len() == path.len() { // no additional points after this step
            true
        } else {
            *path = PathF64::from_points(new_points);
            *corners = new_corners;
            false
        }
    }
    
    /// Calculate the weighted average tangent vector at the tail of 'path'.
    /// Either the last 'n' points, the most number of points at the tail such that the sum of segment
    /// lengths is at most base_length, or the last points until a corner is seen, whichever is the smallest,
    /// are taken into account.
    /// The weights are stronger towards the tail, this is specified by 'tail_weight_multiplier'.
    /// The behavior is undefined unless path is open and 1 < n <= path.len().
    fn calculate_weighted_average_tangent_at_tail(path: PathF64, corners: &[bool], n: usize, base_length: f64, tail_weight_multiplier: f64) -> PointF64 {
        let len = path.len();
        assert!(1 < n);
        assert!(n <= len);

        let mut tangent_acc = PointF64::default();
        let mut length_acc = 0.0;
        let rev_points: Vec<PointF64> = path.path.into_iter().rev().take(n).collect();
        let rev_corners: Vec<&bool> = corners.iter().rev().take(n).collect();
        for (i, point_pair) in rev_points.windows(2).enumerate() {
            // Stop at first corner from tail
            if *rev_corners[i] {
                break;
            }

            let (from, to) = (point_pair[1], point_pair[0]);
            let from_to = to - from;
            tangent_acc *= tail_weight_multiplier; // Stronger weights towards the tail (multiplied more times)
            tangent_acc += from_to.get_normalized();

            length_acc += from_to.norm();
            if length_acc >= base_length {
                break;
            }
        }

        tangent_acc.get_normalized()
    }

    /// Calculate the cubic bezier curve from 'from_point' to 'to_point' with the provided tangents.
    fn calculate_cubic_curve(from_point: PointF64, from_tangent: PointF64, to_point: PointF64, to_tangent: PointF64, smoothness: usize) -> PathF64 {
        let scaled_base_length = from_point.distance_to(to_point) * 2.0;    
        let intersection = calculate_intersection(from_point, from_point + from_tangent, to_point, to_point + to_tangent);

        let length_from_and_intersection = from_point.distance_to(intersection);
        let length_to_and_intersection = to_point.distance_to(intersection);

        let evaluate_control_point = |point: PointF64, tangent:PointF64, length_with_intersection: f64| {
            if scaled_base_length > length_with_intersection * 0.5 {
                calculate_midpoint(point, intersection)
            } else {
                point + tangent.get_normalized() * scaled_base_length
            }
        };
        let control_point1 = evaluate_control_point(from_point, from_tangent, length_from_and_intersection);
        let control_point2 = evaluate_control_point(to_point, to_tangent, length_to_and_intersection);
        
        let curve = bezier::Curve::from_points(
            from_point,
            (control_point1, control_point2),
            to_point
        );
        
        let points: Vec<PointF64> = (0..=smoothness)
                                        .into_iter()
                                        .map(|i| {
                                            let t = i as f64 / smoothness as f64;
                                            curve.point_at_pos(t)
                                        })
                                        .collect();

        PathF64::from_points(points)
    }
}