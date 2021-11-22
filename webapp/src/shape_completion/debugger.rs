use visioncortex::{Color, CompoundPath, PathF64, PathI32, PointF64, PointI32, Spline};

use super::FilledHoleMatrix;

pub trait Debugger {
    fn should_draw_simplified(&self) -> bool;

    fn should_draw_smoothed(&self) -> bool;

    fn should_draw_control_points(&self) -> bool;

    fn should_draw_tail_tangents(&self) -> bool;

    fn fill_rect(&self, color: &Color, x: usize, y: usize, w: usize, h: usize);

    fn draw_pixel_i32(&self, color: &Color, point: PointI32);

    fn draw_cross_i32(&self, color: &Color, center: PointI32);

    fn draw_path_i32(&self, color: &Color, path: &PathI32);

    fn draw_path_f64(&self, color: &Color, path: &PathF64);

    fn draw_line_f64(&self, color: &Color, from: PointF64, to: PointF64);

    fn draw_spline(&self, color: &Color, spline: &Spline);

    fn draw_cubic_bezier_curve(&self, color: &Color, control_points: [PointF64; 4]);

    fn draw_compound_path(&self, color: &Color, compound_path: &CompoundPath);

    fn draw_filled_hole(&self, filled_hole: FilledHoleMatrix, origin: PointI32);

    fn log(&self, msg: &str);
}
