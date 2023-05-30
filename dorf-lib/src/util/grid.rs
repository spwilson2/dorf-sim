use crate::prelude::*;

#[derive(Debug)]
pub struct Grid2D<T> {
    data: Vec<T>,
    rect: Rect2D,
}

impl<T: Clone> Grid2D<T> {
    pub fn new(topleft: IVec2, size: UVec2, fill: T) -> Self {
        Self {
            data: vec![fill; (size.x * size.y) as usize],
            rect: Rect2D::from_corners(topleft, topleft + size.as_ivec2()),
        }
    }
}
impl<T> Grid2D<T> {
    pub fn rect(&self) -> &Rect2D {
        &self.rect
    }
    #[inline]
    pub fn get(&self, point: IVec2) -> Result<&T, LightError> {
        let idx = self.idx_for_point(point)?;
        match self.data.get(idx) {
            Some(t) => Ok(t),
            None => Err(LightError::OutOfBoundsError),
        }
    }
    /// Panics if point out of range
    #[inline]
    pub fn set_idx(&mut self, idx: usize, entity: T) {
        self.data[idx] = entity;
    }
    /// Panics if point out of range
    #[inline]
    pub fn set(&mut self, point: IVec2, entity: T) {
        let idx = self.idx_for_point(point).unwrap();
        self.data[idx] = entity;
    }
    #[inline]
    fn idx_for_point(&self, point: IVec2) -> Result<usize, LightError> {
        if !self.rect.contains_exclusive_max(point.as_vec2()) {
            return Err(LightError::OutOfBoundsError);
        }
        let top_left = self.rect.min;
        // Distance_y * size_x  + Distance_x
        Ok(((top_left.y + point.y) * self.rect.size().x + (top_left.x + point.x)) as usize)
    }
}
