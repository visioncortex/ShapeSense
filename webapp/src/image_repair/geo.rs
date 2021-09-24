use std::f64::consts::PI;

use visioncortex::{PathF64, PointF64};

// Geometry helper functions

pub fn calculate_midpoint(p1: PointF64, p2: PointF64) -> PointF64 {
    p1 * 0.5 + p2 * 0.5
}

// Given lines p1p2 and p3p4, returns their intersection.
// If the two lines coincide, returns the mid-pt of p1 and p4.
// If the two lines are parallel, also returns the mid-pt of p1 and p4.
#[allow(non_snake_case)]
pub fn calculate_intersection(p1: PointF64, p2: PointF64, p3: PointF64, p4: PointF64) -> PointF64 {
    // Find the equation of a straight line defined by 2 points in the form of Ax + By = C.
    let find_line_equation = |a: &PointF64, b: &PointF64| {
        let A = -(a.y - b.y);
        let B = a.x - b.x;
        let C = a.y * (a.x - b.x) - a.x * (a.y - b.y);
        (A, B, C)
    };

    let f64_approximately = |a: f64, b: f64| { (a - b).abs() <= 1e-7 };

    let (A1, B1, C1) = find_line_equation(&p1, &p2);
    let (A2, B2, C2) = find_line_equation(&p3, &p4);

    if f64_approximately(A1/A2, B1/B2) && f64_approximately(B1/B2, C1/C2) {
        return calculate_midpoint(p1, p4);
    }

    let determinant = A1 * B2 - A2 * B1;
    if f64_approximately(determinant, 0.0) {
        return calculate_midpoint(p1, p4);
    }

    let x = (B2 * C1 - B1 * C2) / determinant;
    let y = (A1 * C2 - A2 * C1) / determinant;
    PointF64::new(x, y)
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
/// (angle in radians bigger or equal to than 'threshold').
pub fn find_corners(path: &PathF64, threshold: f64) -> Vec<bool> {
    if path.is_empty() {
        return vec![];
    }
    
    let path = path.to_unclosed();
    let len = path.len();

    let mut corners: Vec<bool> = vec![false; len];
    for i in 1..(len-1) {
        let prev = i-1;
        let next = i+1;

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
    p_i: &PointF64, p_j: &PointF64, p_1: &PointF64, p_2: &PointF64, outset_ratio: f64) -> PointF64 {
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