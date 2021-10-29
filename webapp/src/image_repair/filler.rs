use std::ops::{Index, IndexMut};

use visioncortex::{BinaryImage, BoundingRect, CompoundPath, PointI32};

#[derive(Clone)]
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

/// A class to fill colors into image whose structural information has been recovered.
pub struct HoleFiller;

// API
impl HoleFiller {
   pub fn fill(
       image: &BinaryImage,
       hole_rect: &BoundingRect,
       interpolated_curves: Vec<CompoundPath>,
       endpoints: Vec<PointI32>
    ) -> FilledHoleMatrix {
       let matrix = FilledHoleMatrix::new(hole_rect.width() as usize, hole_rect.height() as usize);

       matrix
   }
}

// Helper functions
impl HoleFiller {

}