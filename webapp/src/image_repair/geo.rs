use std::f64::consts::PI;

use visioncortex::{PathF64, PointF64};

// Geometry helper functions

fn f64_approximately(a: f64, b: f64) -> bool {
    (a - b).abs() <= f64::EPSILON
}

pub fn calculate_midpoint(p1: PointF64, p2: PointF64) -> PointF64 {
    p1 * 0.5 + p2 * 0.5
}

// Given a line p1p2, returns its unit normal at right hand side.
// Note that the negative of the returned vector is the unit normal at left hand side.
pub fn calculate_unit_normal_of_line(p1: PointF64, p2: PointF64) -> PointF64 {
    let (dx, dy) = (p2.x - p1.x, p2.y - p1.y);
    PointF64::new(-dy, dx).get_normalized()
}

// Given directed lines p1p2 and p3p4, returns their intersection only if it is in the positive direction.
// If the two lines coincide or are parallel, returns the mid-pt of p2 and p3.
// If the intersection of the lines is not in the positive direction, returns none.
pub fn calculate_intersection(p1: PointF64, p2: PointF64, p3: PointF64, p4: PointF64) -> Option<PointF64> {
    let extract_coords = |p: &PointF64| {(p.x, p.y)};
    let (x1, y1) = extract_coords(&p1);
    let (x2, y2) = extract_coords(&p2);
    let (x3, y3) = extract_coords(&p3);
    let (x4, y4) = extract_coords(&p4);

    // Calculate u_a and u_b
    // u_a parametrizes p1p2 and u_b parametrizes p3p4
    let denom = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    if f64_approximately(denom, 0.0) { // Parallel check
        return Some(calculate_midpoint(p2, p3));
    }
    let numera_a = (x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3);
    let numera_b = (x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3);
    // All of denom, numera_a and numera_b are 0
    if f64_approximately(denom + numera_a + numera_b, 0.0) { // Coincidence check
        return Some(calculate_midpoint(p2, p3));
    }
    let u_a = numera_a / denom;
    let u_b = numera_b / denom;

    // Positive direction check
    if u_a.is_sign_positive() && u_b.is_sign_positive() {
        Some(PointF64::new(x1 + u_a * (x2 - x1), y1 + u_a * (y2 - y1)))
    } else {
        None
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