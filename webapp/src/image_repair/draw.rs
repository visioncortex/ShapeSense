use visioncortex::{Color, PointI32};
use web_sys::CanvasRenderingContext2d;

use crate::canvas::Canvas;

pub(crate) struct DrawUtil {
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

    pub fn fill_rect(&self, color: Color, x: usize, y: usize, w: usize, h: usize) {
        self.ctx().set_fill_style(&color.to_hex_string().into());
        self.ctx().fill_rect(x as f64, y as f64, w as f64, h as f64);
    }
    
    pub fn draw_pixel(&self, color: Color, point: PointI32) {
        self.fill_rect(color, point.x as usize, point.y as usize, 1, 1)
    }
}