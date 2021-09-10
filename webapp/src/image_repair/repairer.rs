use visioncortex::{BoundingRect, Color, ColorImage, ColorName, PathI32, PointI32, color_clusters::{Clusters, HIERARCHICAL_MAX, Runner, RunnerConfig}};
use wasm_bindgen::prelude::*;

use crate::util::console_log_util;

use super::DrawUtil;

#[wasm_bindgen]
pub struct Repairer {
    draw_util: DrawUtil,
    image: ColorImage,
    hole_rect: BoundingRect,
}

#[wasm_bindgen]
impl Repairer {
    #[wasm_bindgen(constructor)]
    pub fn new_from_canvas_id_and_mask(canvas_id: &str, x: usize, y: usize, w: usize, h: usize) -> Self {
        let draw_util = DrawUtil::new_from_canvas_id(canvas_id);
        let canvas = &draw_util.canvas;

        // Raw image
        let mut image = canvas.get_image_data_as_color_image(0, 0, canvas.width() as u32, canvas.height() as u32);

        let hole_rect = BoundingRect::new_x_y_w_h(x as i32, y as i32, w as i32, h as i32);

        let empty_color = Color::color(&ColorName::White);

        // Remove hole
        for x_offset in 0..hole_rect.width() {
            for y_offset in 0..hole_rect.height() {
                image.set_pixel(x + x_offset as usize, y + y_offset as usize, &empty_color)
            }
        }

        // Draw hole on canvas
        draw_util.fill_rect(empty_color, x, y, w, h);

        Self { draw_util, image, hole_rect }
    }

    pub fn repair(&self) {
        let paths = self.get_paths();

        paths.iter().enumerate().for_each(|(i, path)| {
            console_log_util(format!("{:?}", path.path));
            path.path.iter().for_each(|point| {
                self.draw_util.draw_pixel(Color::get_palette_color(i+1), *point);
                if point.y == 120 &&
                    370 <= point.x && point.x <= 430 {
                    console_log_util(format!("{:?} on boundary of hole!", point));
                }
            });
        });
    }
}

impl Repairer {
    fn get_paths(&self) -> Vec<PathI32> {
        // Clustering
        let runner_config = RunnerConfig {
            diagonal: false,
            hierarchical: 64,
            batch_size: 25600,
            good_min_area: 1,
            good_max_area: self.image.width * self.image.height,
            is_same_color_a: 4,
            is_same_color_b: 1,
            deepen_diff: 0,
            hollow_neighbours: 1,
            key_color: Color::color(&ColorName::White),
        };
        let runner = Runner::new(runner_config, self.image.clone());
        let clusters = runner.run();
        let clusters_view = clusters.view();

        // Path walking
        clusters_view.clusters.iter().filter_map(|cluster| {
            let image = cluster.to_image(&clusters_view);
            let path = PathI32::image_to_path(&image, true, visioncortex::PathSimplifyMode::None);

            if path.len() <= 5 {
                None
            } else {
                // Path is closed by default, but we just want a curve
                let len = path.len();
                let unclosed_path = path.path[0..(len-1)].to_vec();
                let mut path = PathI32 { path: unclosed_path };

                // Apply offset to get coords in original image
                let offset = PointI32::new(cluster.rect.left, cluster.rect.top);
                path.offset(&offset);
                Some(path)
            }
        }).collect()
    }

    fn find_endpoints(&self) {

    }
}