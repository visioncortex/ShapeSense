use std::collections::HashMap;

use visioncortex::{BoundingRect, Color, ColorImage, ColorName, PathI32, PointI32, color_clusters::{Runner, RunnerConfig}};
use wasm_bindgen::prelude::*;

use crate::util::console_log_util;

use super::draw::DrawUtil;

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
        draw_util.fill_rect(&empty_color, x, y, w, h);

        Self { draw_util, image, hole_rect }
    }

    pub fn repair(&self) {
        let paths = self.get_paths();

        // paths.iter().enumerate().for_each(|(i, path)| {
        //     console_log_util(format!("{:?}", path.path));
        //     self.draw_util.draw_path(Color::get_palette_color(i+1), path);
        // });

        let paths_with_two_endpoints: Vec<(PathI32, Vec<usize>)> = paths
            .into_iter()
            .filter_map(|path| {
                let endpoints = self.find_endpoints_on_path(&path);
                if endpoints.len() == 2 {
                    Some((path, endpoints))
                } else {
                    None
                }
            })
            .collect();

        // Assume only 1 path with 2 endpoints
        assert!(paths_with_two_endpoints.len() == 1);
        let (path, endpoints) = &paths_with_two_endpoints[0];

        let head = std::cmp::min(endpoints[0], endpoints[1]);
        let midpoint: usize = endpoints.iter().sum::<usize>() >> 1;
        let tail = std::cmp::max(endpoints[0], endpoints[1]);

        let (curve1, curve2) = Self::split_path(path, head, midpoint, tail);
        self.draw_util.draw_path_i32(&Color::get_palette_color(1), &curve1);
        self.draw_util.draw_path_i32(&Color::get_palette_color(3), &curve2);
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
            let mut path = PathI32::image_to_path(&image, true, visioncortex::PathSimplifyMode::None);

            if path.len() <= 5 {
                None
            } else {
                // Apply offset to get coords in original image
                let offset = PointI32::new(cluster.rect.left, cluster.rect.top);
                path.offset(&offset);
                Some(path)
            }
        }).collect()
    }

    /// Currently naive approach of checking whether points in the path are on the boundary of hole_rect.
    /// Return indices of points on 'path' that are on said boundary.
    fn find_endpoints_on_path(&self, path: &PathI32) -> Vec<usize> {
        let mut points = HashMap::new();

        path.path
            .iter()
            .enumerate()
            .for_each(|(i, &point)| {
                if self.hole_rect.have_point_on_boundary(point) {
                    points.insert(point, i);
                }
            });

        points.into_values().collect()
    }

    /// Get two sections from 'path': [mid..head] and [mid..tail].
    /// Return an tuple of 2 PathI32, which are the two paths in the order above.
    /// The behavior is undefined unless head <= mid <= tail < path.len().
    fn split_path(path: &PathI32, head: usize, mid: usize, tail: usize) -> (PathI32, PathI32) {
        assert!(head <= mid);
        assert!(mid <= tail);
        assert!(tail < path.len());

        let mut head_to_mid = path[head..=mid].to_vec();
        head_to_mid.reverse();
        let mid_to_head = PathI32::from_points(head_to_mid);

        let mid_to_tail = PathI32::from_points(path[mid..=tail].to_vec());

        (mid_to_head, mid_to_tail)
    }
}