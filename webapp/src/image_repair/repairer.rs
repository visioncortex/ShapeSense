use bit_vec::BitVec;
use flo_curves::{BezierCurve, BezierCurveFactory, bezier::{self, Curve}};
use visioncortex::{BoundingRect, Color, ColorImage, ColorName, PathF64, PathI32, PointF64, PointI32, color_clusters::{Runner, RunnerConfig}};
use wasm_bindgen::prelude::*;

use crate::util::console_log_util;

use super::draw::DrawUtil;

#[wasm_bindgen]
pub struct Repairer {
    draw_util: DrawUtil,
    image: ColorImage,
    hole_rect: BoundingRect,
}

#[wasm_bindgen]
impl Repairer {
    #[wasm_bindgen(constructor)]
    pub fn new_from_canvas_id_and_mask(canvas_id: &str, x: usize, y: usize, w: usize, h: usize) -> Self {
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

        Self { draw_util, image, hole_rect }
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

        let endpoint1 = path[tail1].to_point_f64();
        let endpoint2 = path[tail2].to_point_f64();

        let color1 = Color::get_palette_color(1);
        let color2 = Color::get_palette_color(3);

        let (curve1, curve2) = self.split_path(path, tail1, tail2);
        // self.draw_util.draw_path_i32(&color1, &curve1);
        // self.draw_util.draw_path_i32(&color2, &curve2);

        //# Curve simplification
        let tolerance = 2.5;
        let simplified_curve1 = PathI32::from_points(visioncortex::reduce::reduce(&curve1.path, tolerance));
        let simplified_curve2 = PathI32::from_points(visioncortex::reduce::reduce(&curve2.path, tolerance));

        // self.draw_util.draw_path_i32(&color1, &simplified_curve1);
        // self.draw_util.draw_path_i32(&color2, &simplified_curve2);

        // console_log_util(format!("{:?}", simplified_curve1));
        // console_log_util(format!("{:?}", simplified_curve2));

        //# Curve smoothing
        let outset_ratio = 8.0;
        let min_segment_length = 4.0;
        let max_iterations = 5;

        let smooth_curve1 = self.smoothen_open_curve_iterative(simplified_curve1, outset_ratio, min_segment_length, max_iterations);
        let smooth_curve2 = self.smoothen_open_curve_iterative(simplified_curve2, outset_ratio, min_segment_length, max_iterations);

        self.draw_util.draw_path_f64(&color1, &smooth_curve1);
        self.draw_util.draw_path_f64(&color2, &smooth_curve2);

        // console_log_util(format!("{} {}", simplified_curve1.len(), smooth_curve1.len()));
        // console_log_util(format!("{} {}", simplified_curve2.len(), smooth_curve2.len()));

        // console_log_util(format!("{:?}", &smooth_curve1[(smooth_curve1.len()-10)..]));
        // console_log_util(format!("{:?}", &smooth_curve2[(smooth_curve2.len()-10)..]));

        //# Tail tangent approximation
        let tail_tangent_n = 5;
        let tail_tangent1 = Self::calculate_weighted_average_tangent_at_tail(smooth_curve1, tail_tangent_n);
        let tail_tangent2 = Self::calculate_weighted_average_tangent_at_tail(smooth_curve2, tail_tangent_n);

        // let tangent_visual_length = 20.0;
        // self.draw_util.draw_line_f64(&color1, endpoint1, endpoint1 + tail_tangent1*tangent_visual_length);
        // self.draw_util.draw_line_f64(&color2, endpoint2, endpoint2 + tail_tangent2*tangent_visual_length);
        
        //# Curve interpolation
        let smoothness = 100;

        // let quadratic_curve = self.calculate_quadratic_curve(endpoint1, tail_tangent1, endpoint2, tail_tangent2, smoothness);

        let cubic_curve = self.calculate_cubic_curve(endpoint1, tail_tangent1, endpoint2, tail_tangent2, smoothness);

        // self.draw_util.draw_path_f64(&Color::get_palette_color(6), &quadratic_curve);
        self.draw_util.draw_path_f64(&Color::get_palette_color(4), &cubic_curve);
    }
}

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
            let tail_neighbors_idx_dir = vec![(if tail == 0 {len-1} else {tail-1}, -1), (tail+1 % len, 1)];
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

    /// Apply the 4-point scheme subdivision on 'path' in a convolutional manner iteratively.
    /// Segments (at any point during iteration) shorter than 'min_segment_length' are not further subdivided.
    /// If no subdivision is performed, the iterative process is terminated early.
    /// The behavior is undefined unless path.len() >= 4
    fn smoothen_open_curve_iterative(&self, path: PathI32, outset_ratio: f64, min_segment_length: f64, max_iterations: usize) -> PathF64 {
        assert!(path.len() >= 4);
        
        let mut smooth_curve = path.to_path_f64();
        
        for _ in 0..max_iterations {
            let can_terminate_early = self.smoothen_open_curve_step(&mut smooth_curve, outset_ratio, min_segment_length);

            // Early termination
            if can_terminate_early {
                break;
            }
        }

        smooth_curve
    }

    /// Return true if no subdivision is done in this step.
    fn smoothen_open_curve_step(&self, path: &mut PathF64, outset_ratio: f64, min_segment_length: f64) -> bool {
        let mut new_points = vec![path[0]];

        // Duplicate the last point to make sure all segments except the first are taken care of
        path.add(path[path.len()-1]);

        // Apply 4-point scheme on 'path' in a convolutional manner
        for points in path.path.windows(4) {
            new_points.push(points[1]);

            // Threshold on segment length of the segment to be broken down
            let checked_segment_length = (points[1] - points[2]).norm();
            if checked_segment_length >= min_segment_length {
                new_points.push(Self::find_new_point_from_4_point_scheme(
                    &points[1], &points[2], &points[0], &points[3], outset_ratio));
            }    
        }

        // Push the last 2 points
        new_points.extend(
            path.path.iter()
                     .rev() // To get the last
                     .skip(1) // To skip the artificial duplicate point
                     .take(2) // To get the last two points
                     .rev() // In the original order
        );

        if new_points.len() == path.len() { // no additional points after this step
            true
        } else {
            *path = PathF64::from_points(new_points);
            false
        }
    }

    /// Finds mid-points between (p_i and p_j) and (p_1 and p_2), where p_i and p_j should be between p_1 and p_2,
    /// then returns the new point constructed by the 4-point scheme
    fn find_new_point_from_4_point_scheme(
        p_i: &PointF64, p_j: &PointF64, p_1: &PointF64, p_2: &PointF64, outset_ratio: f64
    ) -> PointF64 {

        let mid_out = Self::calculate_midpoint(*p_i, *p_j);
        let mid_in = Self::calculate_midpoint(*p_1, *p_2);

        let vector_out: PointF64 = mid_out - mid_in;
        let new_magnitude = vector_out.norm() / outset_ratio;
        if new_magnitude < 1e-5 {
            // mid_out == mid_in in this case
            return mid_out;
        }
        let unit_vector = vector_out.get_normalized();
        let frac_vector = PointF64 {x: unit_vector.x * new_magnitude, y: unit_vector.y * new_magnitude};

        // Point out from mid_out
        mid_out + frac_vector
    }
    
    /// Calculate the weighted average tangent vector at the tail of 'path'.
    /// The last 'n' points are taken into account.
    /// The weights are stronger towards the tail.
    /// The behavior is undefined unless path is open and 1 < n <= path.len().
    fn calculate_weighted_average_tangent_at_tail(path: PathF64, n: usize) -> PointF64 {
        let len = path.len();
        assert!(1 < n);
        assert!(n <= len);

        let mut tangent_acc = PointF64::default();
        let rev_points: Vec<PointF64> = path.path.into_iter().rev().take(n).collect();
        for point_pair in rev_points.windows(2) {
            let (from, to) = (point_pair[1], point_pair[0]);
            tangent_acc *= 2.0; // Stronger weights towards the end
            tangent_acc += (to - from).get_normalized();
        }

        tangent_acc.get_normalized()
    }

    fn calculate_midpoint(p1: PointF64, p2: PointF64) -> PointF64 {
        let x = (p1.x + p2.x) / 2.0;
        let y = (p1.y + p2.y) / 2.0;
        PointF64::new(x, y)
    }

    // Given lines p1p2 and p3p4, returns their intersection.
    // If the two lines coincide, returns the mid-pt of p1 and p4.
    // If the two lines are parallel, panicks.
    #[allow(non_snake_case)]
    fn calculate_intersection(&self, p1: PointF64, p2: PointF64, p3: PointF64, p4: PointF64) -> PointF64 {
        // Find the equation of a straight line defined by 2 points in the form of Ax + By = C.
        let find_line = |a: &PointF64, b: &PointF64| {
            let A = -(a.y - b.y);
            let B = a.x - b.x;
            let C = a.y * (a.x - b.x) - a.x * (a.y - b.y);
            (A, B, C)
        };

        let f64_approximately = |a: f64, b: f64| { (a - b).abs() <= 1e-7 };

        let find_intersection = |p1: &PointF64, p2: &PointF64, p3: &PointF64, p4: &PointF64| {
            let (A1, B1, C1) = find_line(p1, p2);
            let (A2, B2, C2) = find_line(p3, p4);

            if f64_approximately(A1/A2, B1/B2) && f64_approximately(B1/B2, C1/C2) {
                return Self::calculate_midpoint(*p1, *p4);
            }

            let determinant = A1 * B2 - A2 * B1;
            if f64_approximately(determinant, 0.0) {
                panic!("Parallel lines in find_intersection()!");
            }

            let x = (B2 * C1 - B1 * C2) / determinant;
            let y = (A1 * C2 - A2 * C1) / determinant;

            PointF64::new(x, y)
        };

        let intersection = find_intersection(&p1, &p2, &p3, &p4);
        self.draw_util.draw_pixel_i32(&Color::get_palette_color(5), intersection.to_point_i32());

        intersection
    }

    /// Calculate the cubic bezier curve from 'from_point' to 'to_point' with the provided tangents.
    fn calculate_cubic_curve(&self, from_point: PointF64, from_tangent: PointF64, to_point: PointF64, to_tangent: PointF64, smoothness: usize) -> PathF64 {
        let base_length = (from_point - to_point).norm() * 2.0;    
        let intersection = self.calculate_intersection(from_point, from_point + from_tangent, to_point, to_point + to_tangent);

        let length_from_and_intersection = (from_point - intersection).norm();
        let length_to_and_intersection = (to_point - intersection).norm();

        let control_point1 = if base_length > length_from_and_intersection * 0.5 {
            Self::calculate_midpoint(from_point, intersection)
        } else {
            from_point + from_tangent.get_normalized() * base_length
        };
        let control_point2 = if base_length > length_to_and_intersection * 0.5 {
            Self::calculate_midpoint(to_point, intersection)
        } else {
            to_point + to_tangent.get_normalized() * base_length
        };
        
        let curve = bezier::Curve::from_points(
            from_point,
            (control_point1, control_point2),
            to_point
        );
        
        let points: Vec<PointF64> = (0..smoothness)
            .into_iter()
            .map(|i| {
                let t = i as f64 / smoothness as f64;
                curve.point_at_pos(t)
            })
            .collect();

        PathF64::from_points(points)
    }

    /// Calculate the quadratic bezier curve from 'from_point' to 'to_point' with the provided tangents.
    fn calculate_quadratic_curve(&self, from_point: PointF64, from_tangent: PointF64, to_point: PointF64, to_tangent: PointF64, smoothness: usize) -> PathF64 {
        let intersection = self.calculate_intersection(from_point, from_point + from_tangent, to_point, to_point + to_tangent);
        
        // Find the quadratic Bezier curve
        let mut interpolated_curve = PathF64::new();
        let mut t = 0.0;
        let delta = 1.0 / smoothness as f64;
        let p0 = from_point;
        let p1 = intersection;
        let p2 = to_point;
        while t <= 1.0 {
            let t_inv = 1.0 - t;
            interpolated_curve.add((p0*t_inv + p1*t) * t_inv + (p1*t_inv + p2*t) * t);
            t += delta;
        }
        interpolated_curve.add(to_point);

        interpolated_curve
    }
}