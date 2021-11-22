use std::collections::HashSet;

use bit_vec::BitVec;
use visioncortex::{BinaryImage, BoundingRect, Color, ColorName, CompoundPath, CompoundPathElement, PathI32, PointI32, clusters::Cluster};

use crate::{curve::{CurveIntrapolator, CurveIntrapolatorConfig}, debugger::{Debugger, DummyDebugger}, filler::{FilledHoleMatrix, HoleFiller}, geo::bezier_curves_intersection, matcher::Matcher, matcher_helper::{MatchItem, MatchItemSet, Matching}};

pub struct ShapeCompletor {
    image: BinaryImage,
    simplify_tolerance: f64,
    curve_intrapolator_config: CurveIntrapolatorConfig,
    debugger: Box<dyn Debugger>,
}

// API
impl ShapeCompletor {
    pub fn new(
        image: BinaryImage,
        simplify_tolerance: f64,
        curve_intrapolator_config: CurveIntrapolatorConfig,
        debugger: Option<Box<dyn Debugger>>,
    ) -> Self
    {
        Self {
            image,
            simplify_tolerance,
            curve_intrapolator_config,
            debugger: debugger.unwrap_or_else(|| Box::new(DummyDebugger)),
        }
    }

    pub fn complete_shape_and_draw(&self, hole_rect: BoundingRect) -> Result<(), String> {
        let hole_origin = PointI32::new(hole_rect.left, hole_rect.top);
        let filled_hole = self.complete_shape(hole_rect)?;

        self.debugger.draw_filled_hole(filled_hole, hole_origin);

        Ok(())
    }

    /// If shape completion fails, expand along each side and take the first successful result.
    pub fn complete_shape_and_draw_expandable(
        &self,
        hole_rect: BoundingRect,
    ) -> Result<(), String> {
        let hole_origin = PointI32::new(hole_rect.left, hole_rect.top);
        let filled_hole = match self.complete_shape(hole_rect) {
            Ok(filled_hole) => filled_hole,
            Err(mut error) => {
                error += "\n";
                let try_expand = || {
                    let (x, y, w, h) = (
                        hole_rect.left,
                        hole_rect.top,
                        hole_rect.width(),
                        hole_rect.height(),
                    );
                    let expanded_hole_rects = [
                        BoundingRect::new_x_y_w_h(x - 1, y, w + 1, h), // Expanded to the left
                        BoundingRect::new_x_y_w_h(x, y - 1, w, h + 1), // Expanded upward
                        BoundingRect::new_x_y_w_h(x, y, w + 1, h),     // Expanded to the right
                        BoundingRect::new_x_y_w_h(x, y, w, h + 1),     // Expanded downward
                    ];
                    for (i, &expanded_hole_rect) in expanded_hole_rects.iter().enumerate() {
                        if 0 <= hole_rect.left
                            && hole_rect.right <= self.image.width as i32
                            && 0 <= hole_rect.top
                            && hole_rect.bottom <= self.image.height as i32
                        {
                            match self.complete_shape(expanded_hole_rect) {
                                Ok(filled_hole) => {
                                    return Ok(
                                        // Remove the expanded column/row
                                        match i {
                                            0 => filled_hole.new_without_column(0),
                                            1 => filled_hole.new_without_row(0),
                                            2 => filled_hole.new_without_column(w as usize - 1),
                                            3 => filled_hole.new_without_row(h as usize - 1),
                                            _ => panic!("Impossible."),
                                        },
                                    );
                                }
                                Err(expanded_error) => {
                                    error += &(expanded_error + "\n");
                                }
                            }
                        } else {
                            error += "Expansion out of range.\n";
                        }
                    }
                    Err(error)
                };
                try_expand()?
            }
        };

        self.debugger.draw_filled_hole(filled_hole, hole_origin);

        Ok(())
    }

    pub fn complete_shape(&self, hole_rect: BoundingRect) -> Result<FilledHoleMatrix, String> {
        //# Path walking
        let paths = self.get_test_paths();

        //# Path identification, segmentation, and simplification
        let path_segments = self.find_simplified_segments_from_paths(&hole_rect, paths);

        if path_segments.is_empty() {
            return Ok(FilledHoleMatrix::new(
                hole_rect.width() as usize,
                hole_rect.height() as usize,
            ));
        }

        //# Matching paths
        let match_item_set = self.construct_match_item_set(&path_segments)?;
        let matchings = Matcher::find_all_possible_matchings(match_item_set)?;

        let intrapolated_curves = {
            let try_intrapolation = |correct_tail_tangents| {
                self.try_intrapolate_with_matchings(
                    hole_rect,
                    &matchings,
                    &path_segments,
                    correct_tail_tangents,
                )
            };
            // First try intrapolation without correcting tail tangents
            match try_intrapolation(false).or_else(|| try_intrapolation(true)) {
                Some(curves) => curves,
                None => return Err("Still not intrapolated.".into()),
            }
        };

        let endpoints: Vec<PointI32> = path_segments
            .into_iter()
            .map(|segment| segment[0])
            .collect();

        HoleFiller::fill(&self.image, hole_rect, intrapolated_curves, endpoints)
    }
}

// Helper functions
impl ShapeCompletor {
    fn get_test_paths(&self) -> Vec<PathI32> {
        let clusters = self.image.to_clusters(false);

        clusters
            .into_iter()
            .map(|cluster| {
                let origin = PointI32::new(cluster.rect.left, cluster.rect.top);
                let mut paths = Cluster::image_to_paths(
                    &cluster.to_binary_image(),
                    visioncortex::PathSimplifyMode::None,
                );
                paths.iter_mut().for_each(|path| path.offset(&origin));
                paths
            })
            .flatten()
            .collect()
    }

    fn find_simplified_segments_from_paths(
        &self,
        hole_rect: &BoundingRect,
        paths: Vec<PathI32>,
    ) -> Vec<PathI32> {
        let mut endpoints = HashSet::new();
        paths
            .into_iter()
            .map(|path| {
                self.find_segments_on_path_with_unique_endpoints(hole_rect, path, &mut endpoints)
            })
            .flatten()
            .collect()
    }

    /// Return a vector of *simplified* path segments whose heads are endpoints, pointing outwards from hole_rect.
    /// Segments are walked until 'max_num_points' is reached or another boundary point is reached, whichever happens first.
    fn find_segments_on_path_with_unique_endpoints(
        &self,
        hole_rect: &BoundingRect,
        path: PathI32,
        current_endpoints: &mut HashSet<PointI32>,
    ) -> Vec<PathI32> {
        let path = path.to_open();
        let len = path.len();
        let is_boundary_mask =
            BitVec::from_fn(len, |i| hole_rect.have_point_on_boundary(path[i], 1));

        let endpoints_iter = (0..len).into_iter().filter(|&i| {
            let prev = if i == 0 {len-1} else {i-1};
            let next = (i + 1) % len;

            is_boundary_mask[i] // itself is on boundary
            // If both neighbors are on boundary, it is a degenerate case (corner intersection) where there is no endpoints pair.
            && ((is_boundary_mask[prev] && !is_boundary_mask[next]) || (!is_boundary_mask[prev] && is_boundary_mask[next]))

        });

        endpoints_iter
            .filter_map(|endpoint| {
                let inserted = current_endpoints.insert(path[endpoint]);
                if inserted {
                    match self.walk_segment(&path, endpoint, &is_boundary_mask) {
                        Ok(segment) => Some(segment),
                        Err(error) => panic!("{}", error),
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// The behavior is undefined unless path.len() == is_boundary_mask.len().
    fn walk_segment(
        &self,
        path: &PathI32,
        endpoint_index: usize,
        is_boundary_mask: &BitVec<u32>,
    ) -> Result<PathI32, String> {
        if path.len() != is_boundary_mask.len() {
            return Err("Length of path must be equal to length of boundary mask.".into());
        }

        // Determine direction
        let len = path.len();
        let prev = if endpoint_index == 0 {
            len - 1
        } else {
            endpoint_index - 1
        };
        let next = (endpoint_index + 1) % len;
        if is_boundary_mask[prev] == is_boundary_mask[next] {
            // Only one side can be boundary, not degenerate corner case
            return Err("Only one neighbor can be boundary point.".into());
        }
        let direction = if is_boundary_mask[prev] { 1 } else { -1 };

        // Walk from 'endpoint_index' along 'path' by 'direction'
        // until 'max_num_points' points are in the walked path, or another boundary point is added
        let mut path_segment = PathI32::new();
        let mut endpoint = endpoint_index as i32;
        let len = len as i32;
        loop {
            path_segment.add(path[endpoint as usize]);

            endpoint += direction;
            endpoint = if endpoint >= 0 {
                endpoint % len
            } else {
                len - 1
            };

            if is_boundary_mask[endpoint as usize] {
                path_segment.add(path[endpoint as usize]);
                break;
            }
        }

        // Simplify 'path_segment'
        Ok(PathI32::from_points(visioncortex::reduce::reduce(
            &path_segment.path,
            self.simplify_tolerance,
        )))
    }

    /// The behavior is undefined unless 'path_segments' has an even number of elements.
    /// The behavior is also undefined unless every segment has at least 2 points.
    /// The behavior is also undefined unless all segments have their tails at index 0.
    fn construct_match_item_set(&self, path_segments: &[PathI32]) -> Result<MatchItemSet, String> {
        if path_segments.len() % 2 != 0 {
            return Err("There must be an even number of path segments.".into());
        }

        let match_items_iter = path_segments.iter().map(|segment| {
            assert!(segment.len() >= 2);
            // 0 is tail
            let direction = (segment[0] - segment[1]).to_point_f64().get_normalized();
            MatchItem::new_with_default_id(segment[0].to_point_f64(), direction)
        });
        let mut match_item_set = MatchItemSet::new();
        match_items_iter.for_each(|match_item| match_item_set.push_and_set_id(match_item));
        Ok(match_item_set)
    }

    /// Return true iff one of the matchings is successfully intrapolated
    fn try_intrapolate_with_matchings(
        &self,
        hole_rect: BoundingRect,
        matchings: &[Matching],
        path_segments: &[PathI32],
        correct_tail_tangents: bool, // Not a configuration, but a fail-safe feature
    ) -> Option<Vec<CompoundPath>> {
        let curve_intrapolator = CurveIntrapolator::new(
            self.curve_intrapolator_config,
            hole_rect,
            self.debugger.as_ref(),
        );

        'matching_loop: for matching in matchings.iter() {
            let mut intrapolated_curves = vec![];
            for &(index1, index2) in matching.iter() {
                let (curve1, curve2) = (
                    path_segments[index1].to_path_f64(),
                    path_segments[index2].to_path_f64(),
                );

                if self.debugger.should_draw_simplified() {
                    let color1 = Color::get_palette_color(1);
                    let color2 = Color::get_palette_color(3);
                    self.debugger.draw_path_f64(&color1, &curve1);
                    self.debugger.draw_path_f64(&color2, &curve2);
                }

                if let Some(intrapolated_curve) = curve_intrapolator
                    .intrapolate_curve_between_curves(
                        curve1,
                        curve2,
                        false,
                        false,
                        correct_tail_tangents,
                    )
                {
                    intrapolated_curves.push(intrapolated_curve);
                } else {
                    // A curve cannot be intrapolated, this matching is wrong
                    continue 'matching_loop;
                }
            }
            // Check if any curves intersect with each other
            if bezier_curves_intersection(&intrapolated_curves) {
                continue 'matching_loop;
            }

            if self.debugger.should_draw_control_points() {
                let color = Color::color(&ColorName::Black);
                intrapolated_curves.iter().for_each(|curve| {
                    curve.iter().for_each(|part| {
                        if let CompoundPathElement::Spline(part) = part {
                            self.debugger
                                .draw_cross_i32(&color, part.points[1].to_point_i32());
                            self.debugger
                                .draw_cross_i32(&color, part.points[2].to_point_i32());
                        }
                    });
                });
            }

            // Trust it to be the correct solution
            return Some(intrapolated_curves);
        }

        None
    }
}
