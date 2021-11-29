use visioncortex::{Color, CompoundPath, PathF64, PathI32, PointF64, PointI32, Spline};

use crate::filler::FilledHoleMatrix;

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

#[derive(Default)]
pub struct DummyDebugger;

impl Debugger for DummyDebugger {
    fn should_draw_simplified(&self) -> bool {
        false
    }

    fn should_draw_smoothed(&self) -> bool {
        false
    }

    fn should_draw_control_points(&self) -> bool {
        false
    }

    fn should_draw_tail_tangents(&self) -> bool {
        false
    }

    fn fill_rect(&self, _color: &Color, _x: usize, _y: usize, _w: usize, _h: usize) {}

    fn draw_pixel_i32(&self, _color: &Color, _point: PointI32) {}

    fn draw_cross_i32(&self, _color: &Color, _center: PointI32) {}

    fn draw_path_i32(&self, _color: &Color, _path: &PathI32) {}

    fn draw_path_f64(&self, _color: &Color, _path: &PathF64) {}

    fn draw_line_f64(&self, _color: &Color, _from: PointF64, _to: PointF64) {}

    fn draw_spline(&self, _color: &Color, _spline: &Spline) {}

    fn draw_cubic_bezier_curve(&self, _color: &Color, _control_points: [PointF64; 4]) {}

    fn draw_compound_path(&self, _color: &Color, _compound_path: &CompoundPath) {}

    fn draw_filled_hole(&self, _filled_hole: FilledHoleMatrix, _origin: PointI32) {}

    fn log(&self, msg: &str) {
        log::info!("{}", msg);
    }
}
