use std::{collections::HashSet, convert::TryInto, ops::{Index, IndexMut}};

use flo_curves::{BezierCurve, Coord2, Coordinate2D, bezier::Curve};
use visioncortex::{BinaryImage, BoundingRect, CompoundPath, PointF64, PointI32, PointUsize};

use crate::util::{console_log_debug_util, console_log_util};

#[derive(Clone, PartialEq)]
pub enum FilledHoleElement {
    Blank,
    Structure,
    Texture,
}

pub struct FilledHoleMatrix {
    pub width: usize,
    pub height: usize,
    pub elems: Vec<FilledHoleElement>
}

impl FilledHoleMatrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            elems: vec![FilledHoleElement::Blank; width * height],
        }
    }
}

impl Index<usize> for FilledHoleMatrix {
    type Output = [FilledHoleElement]; // Output a row for further indexing

    fn index(&self, index: usize) -> &Self::Output {
        &self.elems[(index * self.width) .. ((index+1) * self.width)]
    }
}

impl IndexMut<usize> for FilledHoleMatrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.elems[(index * self.width) .. ((index+1) * self.width)]
    }
}

impl Index<PointUsize> for FilledHoleMatrix {
    type Output = FilledHoleElement; // Output a row for further indexing

    fn index(&self, index: PointUsize) -> &Self::Output {
        &self.elems[index.y * self.width + index.x]
    }
}

impl IndexMut<PointUsize> for FilledHoleMatrix {
    fn index_mut(&mut self, index: PointUsize) -> &mut Self::Output {
        &mut self.elems[index.y * self.width + index.x]
    }
}

/// A class to fill colors into image whose structural information has been recovered.
pub struct HoleFiller;

// API
impl HoleFiller {
    /// Return a FilledHoleMatrix representing what is inside the hole after filling.
    /// The behavior is undefined unless the size of 'image' is at least the size
    /// of 'hole_rect'.
    pub fn fill(
       image: &BinaryImage,
       hole_rect: BoundingRect,
       interpolated_curves: Vec<CompoundPath>,
       endpoints: Vec<PointI32>
    ) -> FilledHoleMatrix {
        let matrix = FilledHoleMatrix::new(hole_rect.width() as usize, hole_rect.height() as usize);
        let origin = PointI32::new(hole_rect.left, hole_rect.top);

        let matrix = Self::rasterize_interpolated_curves(matrix, interpolated_curves, origin);

        Self::fill_holes(matrix, image, hole_rect, origin, endpoints)
        // matrix
   }
}

// Helper functions
impl HoleFiller {
    fn rasterize_interpolated_curves(mut matrix: FilledHoleMatrix, curves: Vec<CompoundPath>, origin: PointI32) -> FilledHoleMatrix {
        let offset = -origin;
        curves.into_iter().for_each(|mut compound_path| {
            compound_path.iter_mut().for_each(|path_elem| {
                match path_elem {
                    visioncortex::CompoundPathElement::PathI32(path) => {
                        path.offset(&offset);
                        path.iter().for_each(|point| {
                            let point = PointUsize::new(point.x as usize, point.y as usize);
                            matrix[point] = FilledHoleElement::Structure;
                        })
                    },
                    visioncortex::CompoundPathElement::PathF64(path) => {
                        path.offset(&offset.to_point_f64());
                        path.iter().for_each(|point| {
                            let point = PointUsize::new(point.x as usize, point.y as usize);
                            matrix[point] = FilledHoleElement::Structure;
                        })
                    },
                    visioncortex::CompoundPathElement::Spline(spline) => {
                        spline.offset(&offset.to_point_f64());
                        spline.get_control_points().into_iter().for_each(|points| {
                            Self::rasterize_bezier_curve(
                                &mut matrix,
                                points.try_into().expect("Control points must have 4 elements")
                            );
                        });
                    },
                }
            });
        });

        matrix
    }

    fn rasterize_bezier_curve(matrix: &mut FilledHoleMatrix, control_points: [PointF64; 4]) {
        let points: Vec<Coord2> = control_points.iter().map(|p| {
            Coord2(p.x, p.y)
        }).collect();

        let quantization_levels = std::cmp::max(matrix.width, matrix.height) << 1;
        let curve = Curve {
            start_point: points[0],
            end_point: points[3],
            control_points: (points[1], points[2]),
        };

        for i in 0..quantization_levels {
            let t = i as f64 / quantization_levels as f64;
            let p = curve.point_at_pos(t);
            let clipped_p = PointUsize::new(
                std::cmp::min(p.x() as usize, matrix.width-1),
                std::cmp::min(p.y() as usize, matrix.height-1),
            );
            matrix[clipped_p] = FilledHoleElement::Structure;
        }
    }

    /// The behavior is undefined unless 'offset' is the top-left corner of 'hole_rect' (exactly on its boundary).
    fn fill_holes(mut matrix: FilledHoleMatrix, image: &BinaryImage, hole_rect: BoundingRect, offset: PointI32, endpoints: Vec<PointI32>) -> FilledHoleMatrix {
        let max_depth = matrix.width * matrix.height;
        
        let bounding_points = hole_rect.get_boundary_points_from(endpoints[0], true);
        let num_points = bounding_points.len();
        let mut current_point = 1; // Skipping the first endpoint
        // The middle point between from and to in a cyclic manner.
        // Used to sample the middle point between endpoints.
        let sample_point = |from: usize, to: usize| {
            let cyclic_dist = if to >= from {
                to - from
            } else {
                num_points - (from - to)
            };
            (from + (cyclic_dist >> 1)) % num_points
        };

        let endpoints_set = endpoints.iter().copied().collect::<HashSet<PointI32> >();
        let is_endpoint = |p| { endpoints_set.contains(&p) };
        
        // Check if first segment should be filled by majority voting
        // If so, fill it
        let mut to_fill = {
            let mut total_pixels = 0_usize;
            let mut filled_pixels = 0_usize;
            while !is_endpoint(bounding_points[current_point]) {
                total_pixels += 1;
                let outside_point = hole_rect.get_closest_point_outside(bounding_points[current_point]);
                if image.get_pixel_at_safe(outside_point) {
                    filled_pixels += 1;
                }
                current_point = (current_point + 1) % num_points;
            }
            console_log_util(format!("{} {}", filled_pixels, total_pixels));
            if filled_pixels >= (total_pixels >> 1) {
                let sampled_point = sample_point(0, current_point);
                let inside_point = hole_rect.get_closest_point_inside(bounding_points[sampled_point]);
                
                Self::fill_hole_recursive(&mut matrix, (inside_point - offset).to_point_usize(), max_depth);
                false
            } else {
                true
            }
        };

        // Go to next segment. If previous segment was filled, skip this segment, or vice versa.
        // Repeat this until the first endpoint is seen again.

        matrix
    }

    fn fill_hole_recursive(matrix: &mut FilledHoleMatrix, point: PointUsize, depth: usize) {
        if depth == 0 {
            return;
        }
        if point.x >= matrix.width || point.y >= matrix.height {
            return;
        }
        if matrix[point] != FilledHoleElement::Blank {
            return;
        }

        matrix[point] = FilledHoleElement::Texture;

        Self::fill_hole_recursive(matrix, point + PointUsize::new(1, 0), depth-1);
        Self::fill_hole_recursive(matrix, point + PointUsize::new(0, 1), depth-1);

        if point.x > 0 {
            Self::fill_hole_recursive(matrix, point - PointUsize::new(1, 0), depth-1);
        }

        if point.y > 0 {
            Self::fill_hole_recursive(matrix, point - PointUsize::new(0, 1), depth-1);
        }
    }
}