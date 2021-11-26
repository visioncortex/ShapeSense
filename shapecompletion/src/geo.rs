use std::f64::consts::PI;

use flo_curves::{
    bezier::{curve_intersects_curve_clip, Curve},
    Coord2, Coordinate,
};
use visioncortex::{CompoundPath, CompoundPathElement, PathF64, PointF64, Spline};

// Geometry helper functions

fn f64_approximately(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::EPSILON
}

/// ratio : returned point
///
/// 0 : 'from'
///
/// 0.5 : midpoint
///
/// 1 : 'to'
pub fn calculate_in_between_point(from: PointF64, to: PointF64, ratio: f64) -> PointF64 {
    let dir = to - from;
    from + dir * ratio
}

pub fn calculate_midpoint(p1: PointF64, p2: PointF64) -> PointF64 {
    calculate_in_between_point(p1, p2, 0.5)
}

/// Given a line p1p2, returns its unit normal at right hand side.
/// Note that the negative of the returned vector is the unit normal at left hand side.
pub fn calculate_unit_normal_of_line(p1: PointF64, p2: PointF64) -> PointF64 {
    let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
    PointF64::new(-dy, dx).get_normalized()
}

#[derive(Debug)]
pub enum LineIntersectionResult {
    Intersect(PointF64), // The segments can be extended in the positive directions to intersect at this point
    Parallel,            // No intersection at all
    Coincidence,         // Infinite intersections (extended or not)
    None,                // No intersection in the positive directions
}

// Given directed lines p1p2 and p3p4, returns their intersection result.
pub fn calculate_intersection(
    p1: PointF64,
    p2: PointF64,
    p3: PointF64,
    p4: PointF64,
) -> LineIntersectionResult {
    let extract_coords = |p: &PointF64| (p.x, p.y);
    let (x1, y1) = extract_coords(&p1);
    let (x2, y2) = extract_coords(&p2);
    let (x3, y3) = extract_coords(&p3);
    let (x4, y4) = extract_coords(&p4);

    // Calculate u_a and u_b
    // u_a parametrizes p1p2 and u_b parametrizes p3p4
    let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    let numera_a = (x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3);
    let numera_b = (x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3);
    if f64_approximately(denom, 0.0)
        && f64_approximately(numera_a, 0.0)
        && f64_approximately(numera_b, 0.0)
    {
        return LineIntersectionResult::Coincidence;
    }
    if f64_approximately(denom, 0.0) {
        return LineIntersectionResult::Parallel;
    }
    let u_a = numera_a / denom;
    let u_b = numera_b / denom;

    // Positive direction check
    if u_a.is_sign_positive() && u_b.is_sign_positive() {
        LineIntersectionResult::Intersect(PointF64::new(x1 + u_a * (x2 - x1), y1 + u_a * (y2 - y1)))
    } else {
        LineIntersectionResult::None
    }
}

/// Find the inclined angle of a point in (-pi, pi].
pub fn angle_of_point(p: &PointF64) -> f64 {
    if p.y.is_sign_negative() {
        -p.x.acos()
    } else {
        p.x.acos()
    }
}

/// Given two ordered angles in (-pi,pi], find the signed angle difference between them.
/// Positive in clockwise direction, the 0-degree axis is the positive x axis
pub fn signed_angle_difference(from: &f64, to: &f64) -> f64 {
    let v1 = *from;
    let mut v2 = *to;
    if v1 > v2 {
        v2 += 2.0 * PI;
    }

    let diff = v2 - v1;
    if diff > PI {
        diff - 2.0 * PI
    } else {
        diff
    }
}

/// Takes a path representing an arbitrary curve, returns a vector of bool representing its corners
/// (angle in radians bigger than or equal to 'threshold').
pub fn find_corners(path: &PathF64, threshold: f64) -> Vec<bool> {
    if path.is_empty() {
        return vec![];
    }

    let path = path.to_open();
    let len = path.len();

    let mut corners: Vec<bool> = vec![false; len];
    for i in 1..(len - 1) {
        let prev = i - 1;
        let next = i + 1;

        let v1 = path[i] - path[prev];
        let v2 = path[next] - path[i];

        let angle_v1 = angle_of_point(&v1.get_normalized());
        let angle_v2 = angle_of_point(&v2.get_normalized());

        let angle_diff = signed_angle_difference(&angle_v1, &angle_v2).abs();
        if angle_diff >= threshold {
            corners[i] = true;
        }
    }

    corners
}

/// Finds mid-points between (p_i and p_j) and (p_1 and p_2), where p_i and p_j should be between p_1 and p_2,
/// then returns the new point constructed by the 4-point scheme
pub fn find_new_point_from_4_point_scheme(
    p_i: &PointF64,
    p_j: &PointF64,
    p_1: &PointF64,
    p_2: &PointF64,
    outset_ratio: f64,
) -> PointF64 {
    let mid_out = calculate_midpoint(*p_i, *p_j);
    let mid_in = calculate_midpoint(*p_1, *p_2);

    let vector_out = mid_out - mid_in;
    let new_magnitude = vector_out.norm() / outset_ratio;
    if new_magnitude < 1e-5 {
        // mid_out == mid_in in this case
        return mid_out;
    }

    // Point out from mid_out
    mid_out + vector_out.get_normalized() * new_magnitude
}

/// Determine if any curves in one of the compound paths intersect with another curve in another compound path.
/// Assume that no curves within any single compound path intersect with each other.
/// The behavior is undefined unless all elements in all compound paths are Spline and contain exactly 1 curve.
pub fn bezier_curves_intersection(compound_curves: &[CompoundPath]) -> bool {
    // Assertion
    compound_curves.iter().for_each(|compound_curve| {
        compound_curve.iter().for_each(|curve| {
            if let CompoundPathElement::Spline(curve) = curve {
                assert_eq!(curve.num_curves(), 1);
            } else {
                panic!("Not all compound paths are Spline.");
            }
        })
    });

    let single_spline_to_curve = |spline: &Spline| {
        let coords: Vec<Coord2> = spline
            .points
            .iter()
            .map(|p| Coord2::from_components(&[p.x, p.y]))
            .collect();
        Curve {
            start_point: coords[0],
            end_point: coords[3],
            control_points: (coords[1], coords[2]),
        }
    };

    // Convert to a type that is easier to work with
    let curves_vec: Vec<Vec<Curve<Coord2>>> = compound_curves
        .iter()
        .map(|compound_curve| {
            compound_curve
                .iter()
                .filter_map(|curve| {
                    if let CompoundPathElement::Spline(curve) = curve {
                        Some(single_spline_to_curve(curve))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect();

    let two_curves_intersect = |curve1: &Curve<Coord2>, curve2: &Curve<Coord2>| {
        let base_length = |curve: &Curve<Coord2>| curve.start_point.distance_to(&curve.end_point);
        let accuracy = (base_length(curve1) + base_length(curve2)) * 0.25;

        !curve_intersects_curve_clip(curve1, curve2, accuracy).is_empty()
    };

    // Pair-wise checking of intersection
    // If any pair intersects, return true, else false
    curves_vec.iter().enumerate().any(|(i, curves_i)| {
        curves_i.iter().any(|curve_i| {
            curves_vec.iter().skip(i + 1).any(|curves_j| {
                curves_j
                    .iter()
                    .any(|curve_j| two_curves_intersect(curve_i, curve_j))
            })
        })
    })
}

/// Retract a point towards another point until the supplied predicate returns true or n retractions have been done.
/// The direction is (0: from) -> (1: to).
/// The behavior is undefined unless 0.0 <= retract_ratio <= 1.0
pub fn retract_point<P> (
    mut from: PointF64,
    to: PointF64,
    retract_ratio: f64,
    predicate: P,
    n: Option<usize>,
) -> PointF64
where
    P: Fn(PointF64) -> bool
{
    assert!(0.0 <= retract_ratio);
    assert!(retract_ratio <= 1.0);

    let mut i = n.unwrap_or_default();
    while !predicate(from) && (n.is_none() || i > 0)
    {
        from = calculate_in_between_point(from, to, retract_ratio);
        i -= 1;
    }
    from
}