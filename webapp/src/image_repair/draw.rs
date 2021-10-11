use wasm_bindgen::prelude::*;

use std::convert::TryInto;

use visioncortex::{Color, CompoundPath, PathF64, PathI32, PointF64, PointI32, Spline};
use web_sys::CanvasRenderingContext2d;

use crate::canvas::Canvas;

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum DisplaySelector {
    None,
    Simplified,
    Smoothed,
}

pub struct DrawUtil {
    canvas_id: String,
    pub canvas: Canvas,
    pub display_selector: DisplaySelector,
    pub display_tangents: bool,
    pub display_control_points: bool,
}

impl Clone for DrawUtil {
    fn clone(&self) -> Self {
        Self {
            canvas_id: self.canvas_id.clone(),
            canvas: Canvas::new_from_id(&self.canvas_id).unwrap(),
            display_selector: self.display_selector,
            display_tangents: self.display_tangents,
            display_control_points: self.display_control_points,
        }
    }
}

impl DrawUtil {
    pub fn new(canvas_id: &str, display_selector: DisplaySelector, display_tangents: bool, display_control_points: bool) -> Self {
        Self {
            canvas_id: canvas_id.to_string(),
            canvas: Canvas::new_from_id(canvas_id).unwrap(),
            display_selector,
            display_tangents,
            display_control_points,
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
        self.fill_rect(color, point.x as usize, point.y as usize, 1, 1);
    }

    pub fn draw_cross_i32(&self, color: &Color, center: PointI32) {
        self.fill_rect(color, center.x as usize - 1, center.y as usize, 3, 1);
        self.fill_rect(color, center.x as usize, center.y as usize - 1, 1, 3);
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
        if path.is_empty() {
            return;
        }
        
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

    pub fn draw_spline(&self, color: &Color, spline: &Spline) {
        let control_points_iter = spline.iter().as_slice().windows(4).step_by(3);
        for control_points in control_points_iter {
            self.draw_cubic_bezier_curve(color, control_points.try_into().expect("Control points must have 4 elements"));
        }
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

    pub fn draw_compound_path(&self, color: &Color, compound_path: &CompoundPath) {
        for path in compound_path.iter() {
            match path {
                visioncortex::CompoundPathElement::PathI32(path) => self.draw_path_i32(color, path),
                visioncortex::CompoundPathElement::PathF64(path) => self.draw_path_f64(color, path),
                visioncortex::CompoundPathElement::Spline(spline) => self.draw_spline(color, spline),
            }
        }
    }
}