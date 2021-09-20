use visioncortex::{Color, PathF64, PathI32, PointF64, PointI32};
use web_sys::CanvasRenderingContext2d;

use crate::canvas::Canvas;

pub struct DrawUtil {
    pub canvas: Canvas
}

impl DrawUtil {
    pub fn new_from_canvas_id(canvas_id: &str) -> Self {
        Self {
            canvas: Canvas::new_from_id(canvas_id).unwrap()
        }
    }

    fn ctx(&self) -> &CanvasRenderingContext2d {
        self.canvas.get_rendering_context_2d()
    }

    pub fn fill_rect(&self, color: &Color, x: usize, y: usize, w: usize, h: usize) {
        let ctx = self.ctx();
        ctx.set_fill_style(&color.to_hex_string().into());
        ctx.fill_rect(x as f64, y as f64, w as f64, h as f64);
    }
    
    pub fn draw_pixel_i32(&self, color: &Color, point: PointI32) {
        self.fill_rect(color, point.x as usize, point.y as usize, 1, 1)
    }

    pub fn draw_path_i32(&self, color: &Color, path: &PathI32) {
        let ctx = self.ctx();
        ctx.set_stroke_style(&color.to_hex_string().into());

        ctx.begin_path();
        ctx.move_to(path[0].x as f64, path[0].y as f64);
        path.path.iter().for_each(|&point| {
            ctx.line_to(point.x as f64, point.y as f64);
            ctx.stroke();
        });
    }

    pub fn draw_path_f64(&self, color: &Color, path: &PathF64) {
        let ctx = self.ctx();
        ctx.set_stroke_style(&color.to_hex_string().into());

        ctx.begin_path();
        ctx.move_to(path[0].x, path[0].y);
        path.path.iter().for_each(|&point| {
            ctx.line_to(point.x, point.y);
            ctx.stroke();
        });
    }

    pub fn draw_line_f64(&self, color: &Color, from: PointF64, to: PointF64) {
        let ctx = self.ctx();
        ctx.set_stroke_style(&color.to_hex_string().into());

        ctx.begin_path();
        ctx.move_to(from.x, from.y);
        ctx.line_to(to.x, to.y);
        ctx.stroke();
    }

    pub fn draw_cubic_bezier_curve(&self, color: &Color, control_points: [PointF64; 4]) {
        let ctx = self.ctx();
        ctx.set_stroke_style(&color.to_hex_string().into());

        ctx.begin_path();
        ctx.move_to(control_points[0].x, control_points[0].y);
        ctx.bezier_curve_to(
            control_points[1].x, control_points[1].y,
            control_points[2].x, control_points[2].y,
            control_points[3].x, control_points[3].y,
        );
        ctx.stroke();
    }
}