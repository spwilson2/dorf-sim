use crate::prelude::*;

#[derive(Debug, Clone)]
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
    #[inline]
    pub fn from_parts(data: Vec<T>, rect: Rect2D) -> Self {
        Self { data, rect }
    }
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
    #[inline]
    pub fn get_mut(&mut self, point: IVec2) -> Result<&mut T, LightError> {
        let idx = self.idx_for_point(point)?;
        match self.data.get_mut(idx) {
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
        match self.rect.index_for_point(point) {
            Some(res) => Ok(res),
            None => Err(LightError::OutOfBoundsError),
        }
    }
}
