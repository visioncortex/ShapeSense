use visioncortex::{BoundingRect, Color, CompoundPath, PathF64, PointF64, Spline};

use crate::{debugger::Debugger, geo::{LineIntersectionResult, calculate_in_between_point, calculate_intersection, calculate_midpoint, calculate_unit_normal_of_line, find_corners, find_new_point_from_4_point_scheme, retract_point}};

#[derive(Clone, Copy)]
pub struct CurveIntrapolatorConfig {
    // Smoothing
    pub outset_ratio: f64,
    pub min_segment_length: f64,
    pub smooth_max_iterations: usize,
    pub corner_threshold: f64,
    // Tail tangent approx.
    pub tail_tangent_num_points: usize, // [2, Inf]
    pub tail_weight_multiplier: f64,
    pub control_points_retract_ratio: f64,
}

impl Default for CurveIntrapolatorConfig {
    fn default() -> Self {
        Self {
            outset_ratio: 8.0,
            min_segment_length: 4.0,
            smooth_max_iterations: 2,
            corner_threshold: std::f64::consts::FRAC_PI_2,
            tail_tangent_num_points: 5,
            tail_weight_multiplier: 1.5,
            control_points_retract_ratio: 0.4,
        }
    }
}

/// intrapolate in-between curve given 2 curves
pub struct CurveIntrapolator<'a> {
    pub config: CurveIntrapolatorConfig,
    pub hole_rect: BoundingRect,
    pub debugger: &'a dyn Debugger,
}

// API
impl<'a> CurveIntrapolator<'a> {
    pub fn new(
        config: CurveIntrapolatorConfig,
        hole_rect: BoundingRect,
        debugger: &'a dyn Debugger,
    ) -> Self {
        Self {
            config,
            hole_rect,
            debugger,
        }
    }

    /// intrapolate the imaginary curve between two existing curves.
    /// The endpoints of the intrapolated curve are defined by 'at_tail_curve1' and 'at_tail_curve2'.
    /// If 'at_tail_curve1' is true, the last point of 'curve1' is used as one of the endpoints of the curve, otherwise the first
    /// point (head) of 'curve1' is used. The same goes for 'at_tail_curve2' and 'curve2'.
    pub fn intrapolate_curve_between_curves(
        &self,
        mut curve1: PathF64,
        mut curve2: PathF64,
        at_tail_curve1: bool,
        at_tail_curve2: bool,
        correct_tail_tangents: bool,
    ) -> Option<CompoundPath> {
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

        let (endpoint1, endpoint2) = (curve1[curve1.len() - 1], curve2[curve2.len() - 1]);
        let base_length = endpoint1.distance_to(endpoint2);

        //# Curve smoothing
        let outset_ratio = self.config.outset_ratio;
        let min_segment_length = self.config.min_segment_length;
        let max_iterations = self.config.smooth_max_iterations;
        let corner_threshold = self.config.corner_threshold;

        let (smooth_curve1, corners1) = Self::smooth_open_curve_iterative(
            curve1,
            outset_ratio,
            min_segment_length,
            max_iterations,
            corner_threshold,
        );
        let (smooth_curve2, corners2) = Self::smooth_open_curve_iterative(
            curve2,
            outset_ratio,
            min_segment_length,
            max_iterations,
            corner_threshold,
        );

        if self.debugger.should_draw_smoothed() {
            self.debugger.draw_path_f64(&color1, &smooth_curve1);
            self.debugger.draw_path_f64(&color2, &smooth_curve2);
        }

        //# Tail tangent approximation
        let tail_tangent_n_points = self.config.tail_tangent_num_points;
        let tail_weight_multiplier = self.config.tail_weight_multiplier;
        let (smooth_curve1_len, smooth_curve2_len) = (smooth_curve1.len(), smooth_curve2.len());
        let tail_tangent1 = Self::calculate_weighted_average_tangent_at_tail(
            smooth_curve1,
            &corners1,
            std::cmp::min(tail_tangent_n_points, smooth_curve1_len),
            base_length,
            tail_weight_multiplier,
        );
        let tail_tangent2 = Self::calculate_weighted_average_tangent_at_tail(
            smooth_curve2,
            &corners2,
            std::cmp::min(tail_tangent_n_points, smooth_curve2_len),
            base_length,
            tail_weight_multiplier,
        );

        let (tail_tangent1, tail_tangent2) = if correct_tail_tangents {
            Self::correct_tail_tangents(endpoint1, tail_tangent1, endpoint2, tail_tangent2)
        } else {
            (tail_tangent1, tail_tangent2)
        };

        if self.debugger.should_draw_tail_tangents() {
            let tangent_visual_length =
                (self.hole_rect.width() + self.hole_rect.height()) as f64 / 3.5;
            let to_point1 = endpoint1 + tail_tangent1.get_normalized() * tangent_visual_length;
            let to_point2 = endpoint2 + tail_tangent2.get_normalized() * tangent_visual_length;
            self.debugger.draw_line_f64(&color1, endpoint1, to_point1);
            self.debugger.draw_line_f64(&color2, endpoint2, to_point2);
        }

        //# Curve intrapolation
        self.calculate_whole_curve(
            endpoint1,
            tail_tangent1,
            endpoint2,
            tail_tangent2,
        )
    }
}

// Helper functions
impl<'a> CurveIntrapolator<'a> {
    /// Apply the 4-point scheme subdivision on 'path' in a convolutional manner iteratively, preserving corners.
    /// The corners of the smoothed path are returned as a bool mask.
    /// Segments (at any point during iteration) shorter than 'min_segment_length' are not further subdivided.
    /// If no subdivision is performed, the iterative process is terminated early.
    /// 'path' is returned as-is if path.len() < 4
    fn smooth_open_curve_iterative(
        mut path: PathF64,
        outset_ratio: f64,
        min_segment_length: f64,
        max_iterations: usize,
        corner_threshold: f64,
    ) -> (PathF64, Vec<bool>) {
        let mut corners = find_corners(&path, corner_threshold);

        if path.len() < 4 {
            return (path, corners);
        }

        for _ in 0..max_iterations {
            let can_terminate_early = Self::smooth_open_curve_step(
                &mut path,
                &mut corners,
                outset_ratio,
                min_segment_length,
            );

            // Early termination
            if can_terminate_early {
                break;
            }
        }

        (path, corners)
    }

    /// Return true if no subdivision is done in this step.
    fn smooth_open_curve_step(
        path: &mut PathF64,
        corners: &mut Vec<bool>,
        outset_ratio: f64,
        min_segment_length: f64,
    ) -> bool {
        let mut new_points = vec![path[0]];
        let mut new_corners = vec![corners[0]];

        // Duplicate the last point to make sure all segments except the first are subdivided
        path.add(path[path.len() - 1]);

        // Apply 4-point scheme on 'path' in a convolutional manner
        for (i, points) in path.path.windows(4).enumerate() {
            new_points.push(points[1]);
            new_corners.push(corners[i + 1]);

            // Do not smooth out corners
            if corners[i + 1] || corners[i + 2] {
                continue;
            }

            // Threshold on segment length of the segment to be broken down
            let checked_segment_length = points[1].distance_to(points[2]);
            if checked_segment_length >= min_segment_length {
                new_points.push(find_new_point_from_4_point_scheme(
                    &points[1],
                    &points[2],
                    &points[0],
                    &points[3],
                    outset_ratio,
                ));
                new_corners.push(false); // New point must be a non-corner during subdivision
            }
        }

        // Push the original last point
        new_points.extend(path.iter().rev().take(1));
        new_corners.push(corners[corners.len() - 1]);

        assert_eq!(new_points.len(), new_corners.len());

        if new_points.len() == path.len() {
            // no additional points after this step
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
    fn calculate_weighted_average_tangent_at_tail(
        path: PathF64,
        corners: &[bool],
        n: usize,
        base_length: f64,
        tail_weight_multiplier: f64,
    ) -> PointF64 {
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

    /// Make sure the two tangents do not bend outwards
    fn correct_tail_tangents(
        point1: PointF64,
        mut tangent1: PointF64,
        point2: PointF64,
        mut tangent2: PointF64,
    ) -> (PointF64, PointF64) {
        let correct_tangent =
            |tangent: &mut PointF64, root_point: PointF64, segment_point: PointF64| {
                let mut unit_normal = calculate_unit_normal_of_line(root_point, segment_point); // RHS normal
                let root_to_segment = segment_point - root_point;
                if tangent.dot(root_to_segment).is_sign_negative() {
                    // 'tangent' is bent outwards
                    if tangent.dot(unit_normal).is_sign_negative() {
                        // Use the normal on the other side
                        unit_normal = -unit_normal;
                    }
                    *tangent = unit_normal * root_to_segment.norm();
                }
                // Otherwise, 'tangent' is correct already
            };

        correct_tangent(&mut tangent1, point1, point2);
        correct_tangent(&mut tangent2, point2, point1);

        (tangent1, tangent2)
    }

    fn calculate_whole_curve(
        &self,
        from_point: PointF64,
        from_tangent: PointF64,
        to_point: PointF64,
        to_tangent: PointF64,
    ) -> Option<CompoundPath> {
        let intersection_result = calculate_intersection(
            from_point,
            from_point + from_tangent,
            to_point,
            to_point + to_tangent,
        );

        let mut compound_path = CompoundPath::new();

        match intersection_result {
            LineIntersectionResult::Intersect(_) => {
                // Only 1 big part
                let spline = self.calculate_part_curve(
                    from_point,
                    from_tangent,
                    to_point,
                    to_tangent,
                    intersection_result,
                )?;
                compound_path.add_spline(spline);
            }
            LineIntersectionResult::Parallel => {
                if from_tangent.dot(to_tangent).is_sign_positive() {
                    // Same direction
                    // Only 1 big part
                    let spline = self.calculate_part_curve(
                        from_point,
                        from_tangent,
                        to_point,
                        to_tangent,
                        LineIntersectionResult::Parallel,
                    )?;
                    compound_path.add_spline(spline);
                } else {
                    self.cut_two_curves_and_insert(&mut compound_path, from_point, from_tangent, to_point, to_tangent)?;
                }
            }
            LineIntersectionResult::Coincidence => {
                // Just a straight line
                let line = self.calculate_part_curve(
                    from_point,
                    from_tangent,
                    to_point,
                    to_tangent,
                    LineIntersectionResult::Intersect(calculate_midpoint(from_point, to_point)),
                )?;
                compound_path.add_spline(line);
            }
            LineIntersectionResult::None => {
                self.cut_two_curves_and_insert(&mut compound_path, from_point, from_tangent, to_point, to_tangent)?;
            }
        };

        Some(compound_path)
    }

    fn cut_two_curves_and_insert(&self, compound_path: &mut CompoundPath, point_a: PointF64, tangent_a: PointF64, point_b: PointF64, tangent_b: PointF64) -> Option<()> {
        // S-shape detected
        // Divide into 2 parts and concatenate
        let mid_point = calculate_midpoint(point_a, point_b);
        let normal = calculate_unit_normal_of_line(point_a, point_b);
        // Determine the normal to use (+/-) based on the side of the tangents
        let from_side_normal = if tangent_a.dot(normal) > 0.0 {
            normal
        } else {
            -normal
        };
        let to_side_normal = -from_side_normal;
        // Calculate the two parts of the curve, recalculating the intersections
        let from_side_curve = self.calculate_part_curve(
            point_a,
            tangent_a,
            mid_point,
            from_side_normal,
            LineIntersectionResult::None,
        )?;
        let to_side_curve = self.calculate_part_curve(
            mid_point,
            to_side_normal,
            point_b,
            tangent_b,
            LineIntersectionResult::None,
        )?;

        compound_path.add_spline(from_side_curve);
        compound_path.add_spline(to_side_curve);

        Some(())
    }

    /// Calculate the cubic bezier curve from 'from_point' to 'to_point' with the provided tangents.
    /// 'intersection_result' is only to avoid unnecessary recalculation.
    fn calculate_part_curve(
        &self,
        from_point: PointF64,
        from_tangent: PointF64,
        to_point: PointF64,
        to_tangent: PointF64,
        whole_intersection_result: LineIntersectionResult,
    ) -> Option<Spline> {
        let retract_ratio = self.config.control_points_retract_ratio;
        let retract_predicate = |cp: PointF64| {
            self.hole_rect
                .have_point_on_boundary_or_inside(cp.to_point_i32(), 1)
        };
        let retract_max_n = Some(1000);

        let (control_point1, control_point2) = match whole_intersection_result {
            LineIntersectionResult::Intersect(intersection) => {
                self.evaluate_control_points_with_intersection(from_point, from_tangent, to_point, to_tangent, intersection)
            },
            LineIntersectionResult::Parallel => (
                from_point + from_tangent,
                to_point + to_tangent,
            ),
            LineIntersectionResult::Coincidence => panic!("Part curves do not handle coincidence."),
            LineIntersectionResult::None => {
                // Whole curve has been divided -> recalculate intersection
                let intersection_result = calculate_intersection(
                    from_point,
                    from_point + from_tangent,
                    to_point,
                    to_point + to_tangent,
                );
                match intersection_result {
                    LineIntersectionResult::Intersect(intersection) => {
                        self.evaluate_control_points_with_intersection(from_point, from_tangent, to_point, to_tangent, intersection)
                    }
                    LineIntersectionResult::Parallel => {
                        (from_point + from_tangent, to_point + to_tangent)
                    }
                    LineIntersectionResult::Coincidence => {
                        panic!("Part curves do not handle coincidence.")
                    }
                    LineIntersectionResult::None => return None,
                }
            }
        };

        let (control_point1, control_point2) = (
            retract_point(control_point1, from_point, retract_ratio, retract_predicate, retract_max_n),
            retract_point(control_point2, to_point, retract_ratio, retract_predicate, retract_max_n),
        );
        let mut spline = Spline::new(from_point);
        spline.add(control_point1, control_point2, to_point);
        Some(spline)
    }

    fn evaluate_control_points_with_intersection(
        &self,
        from_point: PointF64,
        from_tangent: PointF64,
        to_point: PointF64,
        to_tangent: PointF64,
        intersection: PointF64
    ) -> (PointF64, PointF64)
    {
        let scaled_base_length = from_point.distance_to(to_point) * 2.0;

        let length_from_and_intersection = from_point.distance_to(intersection);
        let length_to_and_intersection = to_point.distance_to(intersection);

        let calculate_control_point =
            |point: PointF64, tangent: PointF64, length_with_intersection: f64| {
                if scaled_base_length > length_with_intersection * 0.5 {
                    calculate_midpoint(point, intersection)
                } else {
                    point + tangent * scaled_base_length
                }
            };
            
        (
            calculate_control_point(from_point, from_tangent, length_from_and_intersection),
            calculate_control_point(to_point, to_tangent, length_to_and_intersection),
        )
    }
}
