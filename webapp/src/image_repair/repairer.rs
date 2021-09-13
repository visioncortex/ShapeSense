use std::collections::HashMap;

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
        let paths = self.get_paths();

        // paths.iter().enumerate().for_each(|(i, path)| {
        //     console_log_util(format!("{:?}", path.path));
        //     self.draw_util.draw_path(Color::get_palette_color(i+1), path);
        // });

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
        let (path, endpoints) = &paths_with_two_endpoints[0];

        let head = std::cmp::min(endpoints[0], endpoints[1]);
        let midpoint: usize = endpoints.iter().sum::<usize>() >> 1;
        let tail = std::cmp::max(endpoints[0], endpoints[1]);

        let color1 = Color::get_palette_color(1);
        let color2 = Color::get_palette_color(3);

        let (curve1, curve2) = Self::split_path(path, head, midpoint, tail);
        // self.draw_util.draw_path_i32(&color1, &curve1);
        // self.draw_util.draw_path_i32(&color2, &curve2);

        let tolerance = 0.7;
        let mut simplified_curve1 = PathI32::from_points(visioncortex::reduce::reduce(&curve1.path, tolerance));
        let mut simplified_curve2 = PathI32::from_points(visioncortex::reduce::reduce(&curve2.path, tolerance));

        // Close the path for smooth function
        simplified_curve1.add(simplified_curve1[0]);
        simplified_curve2.add(simplified_curve2[0]);

        // self.draw_util.draw_path_i32(&color1, &simplified_curve1);
        // self.draw_util.draw_path_i32(&color2, &simplified_curve2);

        // console_log_util(format!("{:?}", simplified_curve1));
        // console_log_util(format!("{:?}", simplified_curve2));

        let corner_threshold = std::f64::consts::FRAC_PI_2;
        let outset_ratio = 20.0;
        let segment_length = 0.33;
        let max_iterations = 5;

        let mut smooth_curve1 = simplified_curve1.smooth(corner_threshold, outset_ratio, segment_length, max_iterations);
        let mut smooth_curve2 = simplified_curve2.smooth(corner_threshold, outset_ratio, segment_length, max_iterations);

        // Remove the last point to make them open curves
        smooth_curve1.pop();
        smooth_curve2.pop();

        // self.draw_util.draw_path_f64(&color1, &smooth_curve1);
        // self.draw_util.draw_path_f64(&color2, &smooth_curve2);

        // console_log_util(format!("{} {}", simplified_curve1.len(), smooth_curve1.len()));
        // console_log_util(format!("{} {}", simplified_curve2.len(), smooth_curve2.len()));

        // console_log_util(format!("{:?}", &smooth_curve1[(smooth_curve1.len()-5)..]));
        // console_log_util(format!("{:?}", &smooth_curve2[(smooth_curve2.len()-5)..]));

        let tail_gradient_n = 10;
        let weight_decay_factor = 0.5;
        let tail_tangent1 = Self::calculate_weighted_average_tangent_at_tail(&smooth_curve1, tail_gradient_n, weight_decay_factor);
        let tail_tangent2 = Self::calculate_weighted_average_tangent_at_tail(&smooth_curve2, tail_gradient_n, weight_decay_factor);

        let endpoint1 = path[head].to_point_f64();
        let endpoint2 = path[tail].to_point_f64();
        // let visual_length = 10.0;
        // self.draw_util.draw_line_f64(&color1, endpoint1, endpoint1 + tail_tangent1*visual_length);
        // self.draw_util.draw_line_f64(&color2, endpoint2, endpoint2 + tail_tangent2*visual_length);

        let smoothness = 300;
        let interpolated_curve = Self::interpolate_curve(endpoint1, tail_tangent1, endpoint2, tail_tangent2, smoothness);
        self.draw_util.draw_path_f64(&Color::get_palette_color(4), &interpolated_curve);

        console_log_util(format!("{:?}", interpolated_curve));
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
    fn find_endpoints_on_path(&self, path: &PathI32) -> Vec<usize> {
        let mut points = HashMap::new();

        path.path
            .iter()
            .enumerate()
            .for_each(|(i, &point)| {
                if self.hole_rect.have_point_on_boundary(point) {
                    points.insert(point, i);
                }
            });

        points.into_values().collect()
    }

    /// Get two sections from 'path': [mid..head] and [mid..tail].
    /// Return an tuple of 2 PathI32, which are the two paths in the order above.
    /// The behavior is undefined unless head <= mid <= tail < path.len().
    fn split_path(path: &PathI32, head: usize, mid: usize, tail: usize) -> (PathI32, PathI32) {
        assert!(head <= mid);
        assert!(mid <= tail);
        assert!(tail < path.len());

        let mut head_to_mid = path[head..=mid].to_vec();
        head_to_mid.reverse();
        let mid_to_head = PathI32::from_points(head_to_mid);

        let mid_to_tail = PathI32::from_points(path[mid..=tail].to_vec());

        (mid_to_head, mid_to_tail)
    }

    /// Calculate the weighted average tangent vector at the tail of 'path'.
    /// The last 'n' points are taken into account.
    /// The tangents closer to the tail receive higher weightings.
    /// A smaller 'weight_decay_factor' indicates a faster decay *farther* away from the tail.
    /// The behavior is undefined unless path is open and 1 < n <= path.len().
    fn calculate_weighted_average_tangent_at_tail(path: &PathF64, n: usize, weight_decay_factor: f64) -> PointF64 {
        let len = path.len();
        assert!(1 < n);
        assert!(n <= len);

        let mut tangent_acc = PointF64::default();
        let mut from = path[len-n];
        for i in (len-n+1)..len {
            let to = path[i];
            tangent_acc = (tangent_acc + (to - from).get_normalized()) * weight_decay_factor;
            from = to;
        }

        tangent_acc.get_normalized()
    }

    /// Interpolate the curve from 'from_point' to 'to_point' with the provided tangents.
    /// 'smoothness' is an unbounded parameter that governs the smoothness of the interpolated curve.
    /// The behavior is undefined unless 0 < smoothness.
    fn interpolate_curve(from_point: PointF64, from_tangent: PointF64, to_point: PointF64, to_tangent: PointF64, smoothness: usize) -> PathF64 {
        let find_mid_point = |p1: &PointF64, p2: &PointF64| {
            let x = (p1.x + p2.x) / 2.0;
            let y = (p1.y + p2.y) / 2.0;
            PointF64::new(x, y)
        };

        // Given lines p1p2 and p3p4, returns their intersection.
        // If the two lines coincide, returns the mid-pt of p2 and p3.
        // If the two lines are parallel, panicks.
        // https://github.com/tyt2y3/vaser-unity/blob/master/Assets/Vaser/Vec2Ext.cs#L107 (Intersect)
        let find_intersection = |p1: &PointF64, p2: &PointF64, p3: &PointF64, p4: &PointF64| {

            const EPSILON: f64 = 1e-7;
            
            let (denom, numera, numerb);
            denom  = (p4.y-p3.y) * (p2.x-p1.x) - (p4.x-p3.x) * (p2.y-p1.y);
            numera = (p4.x-p3.x) * (p1.y-p3.y) - (p4.y-p3.y) * (p1.x-p3.x);
            numerb = (p2.x-p1.x) * (p1.y-p3.y) - (p2.y-p1.y) * (p1.x-p3.x);

            if denom <= EPSILON && numera <= EPSILON && numerb <= EPSILON {
                // The two lines coincide
                return find_mid_point(p2, p3);
            }

            if denom <= EPSILON {
                panic!("The two lines are parallel!");
            }

            let mua = numera/denom;

            PointF64::new(p1.x + mua * (p2.x-p1.x), p1.y + mua * (p2.y-p1.y))
        };

        let intersection = find_intersection(&from_point, &(from_point+from_tangent), &to_point, &(to_point+to_tangent));

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

#[cfg(test)]
mod tests {
    use super::*;

    fn point_f64_approx(a: PointF64, b: PointF64) -> bool {
        (a-b).norm() < 1e-5
    }

    #[test]
    fn tail_gradient() {
        let mut path = PathF64::new();
        path.add(PointF64::new(0.5, 0.0));
        path.add(PointF64::new(1.0, 2.0));
        path.add(PointF64::new(2.0, 4.0));
        assert!(point_f64_approx(PointF64::new(1.0, 2.0).get_normalized(), Repairer::calculate_weighted_average_tangent_at_tail(&path, 2, 0.5)));
        assert!(point_f64_approx(PointF64::new(0.3810091792, 0.9245712548), Repairer::calculate_weighted_average_tangent_at_tail(&path, 3, 0.5)));
    }
}