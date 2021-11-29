use visioncortex::BoundingRect;
use wasm_bindgen::prelude::*;

use shapecompletion::completor::ShapeCompletor;

use crate::shape_completion::ShapeCompletorAPIConfig;

use super::draw::DrawUtil;

#[wasm_bindgen]
pub struct ShapeCompletorAPI;

#[wasm_bindgen]
impl ShapeCompletorAPI {
    pub fn complete_shape_with_config(config: ShapeCompletorAPIConfig) {
        let draw_util = DrawUtil::new(
            config.canvas_id(),
            config.display_selector,
            config.display_tangents,
            config.display_control_points,
        );
        let canvas = &draw_util.canvas;

        // Raw image
        let mut image = canvas
            .get_image_data_as_color_image(0, 0, canvas.width() as u32, canvas.height() as u32)
            .to_binary_image(|c| c.r as usize > c.g as usize + c.b as usize);

        let (x, y, w, h) = (
            config.hole_left,
            config.hole_top,
            config.hole_width,
            config.hole_height,
        );

        let hole_rect = BoundingRect::new_x_y_w_h(x as i32, y as i32, w as i32, h as i32);

        // Remove hole from image
        for x_offset in 0..hole_rect.width() {
            for y_offset in 0..hole_rect.height() {
                image.set_pixel(x + x_offset as usize, y + y_offset as usize, false);
            }
        }

        let shape_completor = ShapeCompletor::new(
            image,
            config.simplify_tolerance,
            config.curve_intrapolator_config(),
            config.filler_blank_boundary_pixels_tolerance,
            Some(Box::new(draw_util)),
        );

        let result = shape_completor.complete_shape_and_draw_expandable(hole_rect);

        match result {
            Ok(_) => {}
            Err(error) => {
                panic!("{}", error)
            }
        }
    }
}