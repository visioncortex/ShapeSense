use std::collections::HashSet;

use bit_vec::BitVec;
use visioncortex::{BinaryImage, BoundingRect, Color, ColorName, CompoundPath, CompoundPathElement, PathI32, PointI32, clusters::Cluster};
use wasm_bindgen::prelude::*;

use crate::{image_repair::{CurveInterpolator, CurveInterpolatorConfig, MatchItem, MatchItemSet, Matcher, bezier_curves_intersection}, util::{console_log_debug_util, console_log_util}};

use super::{HoleFiller, Matching, RepairerConfig, draw::{DisplaySelector, DrawUtil}};

#[wasm_bindgen]
pub struct Repairer {
    image: BinaryImage,
    hole_rect: BoundingRect,
    draw_util: DrawUtil,
}

// WASM API
#[wasm_bindgen]
impl Repairer {
    pub fn repair_with_config(config: RepairerConfig) {
        let draw_util = DrawUtil::new(config.get_canvas_id(), config.display_selector, config.display_tangents, config.display_control_points);
        let canvas = &draw_util.canvas;

        // Raw image
        let mut image = canvas.get_image_data_as_color_image(0, 0, canvas.width() as u32, canvas.height() as u32).to_binary_image(|c| {
            c.r as usize > c.g as usize + c.b as usize
        });

        let (x, y, w, h) = (config.hole_left, config.hole_top, config.hole_width, config.hole_height);

        let hole_rect = BoundingRect::new_x_y_w_h(x as i32, y as i32, w as i32, h as i32);

        let empty_color = Color::color(&ColorName::White);

        // Remove hole from image
        for x_offset in 0..hole_rect.width() {
            for y_offset in 0..hole_rect.height() {
                image.set_pixel(x + x_offset as usize, y + y_offset as usize, false)
            }
        }

        // Draw hole on canvas
        draw_util.fill_rect(&empty_color, x, y, w, h);

        let repairer = Self { image, hole_rect, draw_util };

        repairer.repair(config.simplify_tolerance, config.curve_interpolator_config);
    }
}

// API
impl Repairer {
    pub fn repair(&self, simplify_tolerance: f64, curve_interpolator_config: CurveInterpolatorConfig) {
        //# Path walking
        let paths = self.get_test_paths();

        //# Path identification, segmentation, and simplification
        let path_segments = self.find_simplified_segments_from_paths(paths, simplify_tolerance);

        if path_segments.is_empty() {
            return;
        }

        //# Matching paths
        let match_item_set = self.construct_match_item_set(&path_segments);
        let matchings = Matcher::find_all_possible_matchings(match_item_set);

        let mut correct_tail_tangents = false; // First try interpolation without correcting tail tangents
        let interpolated_curves = self.try_interpolate_with_matchings(
            &matchings,
            &path_segments,
            curve_interpolator_config,
            correct_tail_tangents
        ).unwrap_or_else(|| {
            correct_tail_tangents = true;
            self.try_interpolate_with_matchings(
                &matchings,
                &path_segments,
                curve_interpolator_config,
                correct_tail_tangents
            ).unwrap_or_else(|| panic!("Still not interpolated."))
        });

        // interpolated_curves.into_iter().for_each(|curve| {
        //     self.draw_util.draw_compound_path(&Color::color(&ColorName::Red), &curve)
        // });

        let endpoints: Vec<PointI32> = path_segments
            .into_iter()
            .map(|segment| segment[0] )
            .collect();

        let filled_hole = HoleFiller::fill(&self.image, self.hole_rect, interpolated_curves, endpoints);
        self.draw_util.draw_filled_hole(filled_hole, PointI32::new(self.hole_rect.left, self.hole_rect.top));
    }
}

// Helper functions
impl Repairer {
    // Assume object shape is red
    fn get_test_paths(&self) -> Vec<PathI32> {        
        let clusters = self.image.to_clusters(false);

        clusters
            .into_iter()
            .map(|cluster| {
                let origin = PointI32::new(cluster.rect.left, cluster.rect.top);
                let mut paths = Cluster::image_to_paths(&cluster.to_binary_image(), visioncortex::PathSimplifyMode::None);
                paths
                    .iter_mut()
                    .for_each(|path| { path.offset(&origin) });
                paths
            })
            .flatten()
            .collect()
    }

    // The larger the tolerance, the fewer points will be left in output path.
    fn find_simplified_segments_from_paths(&self, paths: Vec<PathI32>, simplify_tolerance: f64) -> Vec<PathI32> {
        let mut endpoints = HashSet::new();
        paths
            .into_iter()
            .map(|path| {
                self.find_segments_on_path_with_unique_endpoints(path, &mut endpoints, simplify_tolerance)
            })
            .flatten()
            .collect()
    }

    /// Return a vector of *simplified* path segments whose heads are endpoints, pointing outwards from hole_rect.
    /// Segments are walked until 'max_num_points' is reached or another boundary point is reached, whichever happens first.
    fn find_segments_on_path_with_unique_endpoints(&self, path: PathI32, current_endpoints: &mut HashSet<PointI32>, simplify_tolerance: f64) -> Vec<PathI32> {
        let path = path.to_open();
        let len = path.len();
        let is_boundary_mask = BitVec::from_fn(len, |i| {
            self.hole_rect.have_point_on_boundary(path[i], 1)
        });

        let endpoints_iter = (0..len).into_iter().filter(|&i| {
            let prev = if i == 0 {len-1} else {i-1};
            let next = (i + 1) % len;

            is_boundary_mask[i] // itself is on boundary
            // If both neighbors are on boundary, it is a degenerate case (corner intersection) where there is no endpoints pair.
            && ((is_boundary_mask[prev] && !is_boundary_mask[next]) || (!is_boundary_mask[prev] && is_boundary_mask[next]))

        });

        endpoints_iter.filter_map(|endpoint| {
            let inserted = current_endpoints.insert(path[endpoint]);
            if inserted {
                Some(self.walk_segment(&path, endpoint, &is_boundary_mask, simplify_tolerance))
            } else {
                None
            }
        }).collect()
    }

    /// The behavior is undefined unless path.len() == is_boundary_mask.len().
    fn walk_segment(&self, path: &PathI32, endpoint_index: usize, is_boundary_mask: &BitVec<u32>, simplify_tolerance: f64) -> PathI32 {
        assert_eq!(path.len(), is_boundary_mask.len());

        // Determine direction
        let len = path.len();
        let prev = if endpoint_index == 0 {len-1} else {endpoint_index-1};
        let next = (endpoint_index + 1) % len;
        assert!(is_boundary_mask[prev] != is_boundary_mask[next]); // Only one side is boundary, not degenerate corner case
        let direction = if is_boundary_mask[prev] {1} else {-1};

        // Walk from 'endpoint_index' along 'path' by 'direction'
        // until 'max_num_points' points are in the walked path, or another boundary point is added
        let mut path_segment = PathI32::new();
        let mut endpoint = endpoint_index as i32;
        let len = len as i32;
        loop {
            path_segment.add(path[endpoint as usize]);

            endpoint += direction;
            endpoint = if endpoint >= 0 {endpoint % len} else {len-1};

            if is_boundary_mask[endpoint as usize] {
                path_segment.add(path[endpoint as usize]);
                break;
            }
        }

        // Simplify 'path_segment'
        PathI32::from_points(visioncortex::reduce::reduce(&path_segment.path, simplify_tolerance))
    }

    /// The behavior is undefined unless 'path_segments' has an even number of elements.
    /// The behavior is also undefined unless every segment has at least 2 points.
    /// The behavior is also undefined unless all segments have their tails at index 0.
    fn construct_match_item_set(&self, path_segments: &[PathI32]) -> MatchItemSet {
        assert_eq!(path_segments.len() % 2, 0);
        let match_items_iter = path_segments
            .iter()
            .map(|segment| {
                assert!(segment.len() >= 2);
                // 0 is tail
                let direction = (segment[0] - segment[1]).to_point_f64().get_normalized();
                MatchItem::new_with_default_id(segment[0].to_point_f64(), direction)
            });
        let mut match_item_set = MatchItemSet::new();
        match_items_iter.for_each(|match_item| {match_item_set.push_and_set_id(match_item)});
        match_item_set
    }

    /// Return true iff one of the matchings is successfully interpolated
    fn try_interpolate_with_matchings(
        &self,
        matchings: &[Matching],
        path_segments: &[PathI32],
        curve_interpolator_config: CurveInterpolatorConfig,
        correct_tail_tangents: bool // Not a configuration, but a fail-safe feature
    ) -> Option<Vec<CompoundPath> > {
        let curve_interpolator = CurveInterpolator::new(curve_interpolator_config, self.hole_rect, self.draw_util.clone());

        'matching_loop: for matching in matchings.iter() {
            let mut interpolated_curves = vec![];
            for &(index1, index2) in matching.iter() {
                let (curve1, curve2) = (path_segments[index1].to_path_f64(), path_segments[index2].to_path_f64());

                if self.draw_util.display_selector == DisplaySelector::Simplified {
                    let color1 = Color::get_palette_color(1);
                    let color2 = Color::get_palette_color(3);
                    self.draw_util.draw_path_f64(&color1, &curve1);
                    self.draw_util.draw_path_f64(&color2, &curve2);
                }
                
                if let Some(interpolated_curve) = curve_interpolator.interpolate_curve_between_curves(curve1, curve2, false, false, correct_tail_tangents) {
                    interpolated_curves.push(interpolated_curve);
                } else {
                    // A curve cannot be interpolated, this matching is wrong
                    continue 'matching_loop;
                }
            }
            // Check if any curves intersect with each other
            if bezier_curves_intersection(&interpolated_curves) {
                continue 'matching_loop;
            }

            if self.draw_util.display_control_points {
                let color = Color::color(&ColorName::Black);
                interpolated_curves.iter().for_each(|curve| {
                    curve.iter().for_each(|part| {
                        if let CompoundPathElement::Spline(part) = part {
                            self.draw_util.draw_cross_i32(&color, part.points[1].to_point_i32());
                            self.draw_util.draw_cross_i32(&color, part.points[2].to_point_i32());
                        }
                    });
                });
            }
            
            // Trust it to be the correct solution
            return Some(interpolated_curves);
        }

        None
    }
}
