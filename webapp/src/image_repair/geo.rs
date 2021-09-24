use visioncortex::PointF64;

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