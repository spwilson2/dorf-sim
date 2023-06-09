use crate::prelude::*;

/// A rectangle defined by two opposite corners.
///
/// The rectangle is axis aligned, and defined by its minimum and maximum coordinates,
/// stored in `Rect::min` and `Rect::max`, respectively. The minimum/maximum invariant
/// must be upheld by the user when directly assigning the fields, otherwise some methods
/// produce invalid results. It is generally recommended to use one of the constructor
/// methods instead, which will ensure this invariant is met, unless you already have
/// the minimum and maximum corners.
#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Rect2D {
    /// The minimum corner point of the rect.
    pub min: IVec2,
    /// The maximum corner point of the rect.
    pub max: IVec2,
}

#[inline]
pub fn tile_from_vec2(mut vec2: Vec2) -> IVec2 {
    // Upper-left
    vec2.x = vec2.x.floor();
    vec2.y = vec2.y.floor();
    vec2.as_ivec2()
}

impl Rect2D {
    /// Create a new rectangle from two corner points.
    ///
    /// The two points do not need to be the minimum and/or maximum corners.
    /// They only need to be two opposite corners.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::Rect;
    /// let r = Rect::new(0., 4., 10., 6.); // w=10 h=2
    /// let r = Rect::new(2., 3., 5., -1.); // w=3 h=4
    /// ```
    #[inline]
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        Self::from_corners(IVec2::new(x0, y0), IVec2::new(x1, y1))
    }

    /// Create a new rectangle from two corner points.
    ///
    /// The two points do not need to be the minimum and/or maximum corners.
    /// They only need to be two opposite corners.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// // Unit rect from [0,0] to [1,1]
    /// let r = Rect::from_corners(Vec2::ZERO, Vec2::ONE); // w=1 h=1
    /// // Same; the points do not need to be ordered
    /// let r = Rect::from_corners(Vec2::ONE, Vec2::ZERO); // w=1 h=1
    /// ```
    #[inline]
    pub fn from_corners(p0: IVec2, p1: IVec2) -> Self {
        Rect2D {
            min: p0.min(p1),
            max: p0.max(p1),
        }
    }

    #[inline]
    pub fn from_transform2d(transform: &Transform2D) -> Self {
        let p0 = tile_from_vec2(transform.loc.xy());
        // Shift any remaining size to the topleft whole number coordinate.
        Rect2D::from_corners(p0, p0 + transform.scale.as_ivec2())
    }

    /// Create a new rectangle from its center and size.
    ///
    /// # Panics
    ///
    /// This method panics if any of the components of the size is negative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::from_center_size(Vec2::ZERO, Vec2::ONE); // w=1 h=1
    /// assert!(r.min.abs_diff_eq(Vec2::splat(-0.5), 1e-5));
    /// assert!(r.max.abs_diff_eq(Vec2::splat(0.5), 1e-5));
    /// ```
    #[inline]
    pub fn from_center_size(origin: Vec2, size: Vec2) -> Self {
        assert!(size.cmpge(Vec2::ZERO).all());
        let half_size = size / 2.;
        Self::from_center_half_size(origin, half_size)
    }

    /// Create a new rectangle from its center and half-size.
    ///
    /// # Panics
    ///
    /// This method panics if any of the components of the half-size is negative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::from_center_half_size(Vec2::ZERO, Vec2::ONE); // w=2 h=2
    /// assert!(r.min.abs_diff_eq(Vec2::splat(-1.), 1e-5));
    /// assert!(r.max.abs_diff_eq(Vec2::splat(1.), 1e-5));
    /// ```
    #[inline]
    pub fn from_center_half_size(origin: Vec2, half_size: Vec2) -> Self {
        todo!()
        //assert!(half_size.cmpge(Vec2::ZERO).all());
        //Self {
        //    min: origin - half_size,
        //    max: origin + half_size,
        //}
    }

    /// Check if the rectangle is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::from_corners(Vec2::ZERO, Vec2::new(0., 1.)); // w=0 h=1
    /// assert!(r.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.cmpge(self.max).any()
    }

    /// Rectangle width (max.x - min.x).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::Rect;
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// assert!((r.width() - 5.).abs() <= 1e-5);
    /// ```
    #[inline]
    pub fn width(&self) -> f32 {
        todo!()
        //self.max.x - self.min.x
    }

    /// Rectangle height (max.y - min.y).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::Rect;
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// assert!((r.height() - 1.).abs() <= 1e-5);
    /// ```
    #[inline]
    pub fn height(&self) -> i32 {
        self.max.y - self.min.y
    }

    /// Rectangle size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// assert!(r.size().abs_diff_eq(Vec2::new(5., 1.), 1e-5));
    /// ```
    #[inline]
    pub fn size(&self) -> IVec2 {
        self.max - self.min
    }

    /// Rectangle half-size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// assert!(r.half_size().abs_diff_eq(Vec2::new(2.5, 0.5), 1e-5));
    /// ```
    #[inline]
    pub fn half_size(&self) -> Vec2 {
        todo!()
        //self.size() * 0.5
    }

    /// The center point of the rectangle.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// assert!(r.center().abs_diff_eq(Vec2::new(2.5, 0.5), 1e-5));
    /// ```
    #[inline]
    pub fn center(&self) -> Vec2 {
        (self.min.as_vec2() + self.max.as_vec2()) * 0.5
    }

    /// Check if a point lies within this rectangle, inclusive of its edges.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::Rect;
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// assert!(r.contains(r.center()));
    /// assert!(r.contains(r.min));
    /// assert!(r.contains(r.max));
    /// ```
    #[inline]
    pub fn contains(&self, point: Vec2) -> bool {
        (point.cmpge(self.min.as_vec2()) & point.cmple(self.max.as_vec2())).all()
    }

    #[inline]
    pub fn contains_exclusive_max(&self, point: Vec2) -> bool {
        (point.cmpge(self.min.as_vec2()) & point.cmplt(self.max.as_vec2())).all()
    }

    /// Build a new rectangle formed of the union of this rectangle and another rectangle.
    ///
    /// The union is the smallest rectangle enclosing both rectangles.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r1 = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// let r2 = Rect::new(1., -1., 3., 3.); // w=2 h=4
    /// let r = r1.union(r2);
    /// assert!(r.min.abs_diff_eq(Vec2::new(0., -1.), 1e-5));
    /// assert!(r.max.abs_diff_eq(Vec2::new(5., 3.), 1e-5));
    /// ```
    #[inline]
    pub fn union(&self, other: Rect2D) -> Rect2D {
        Rect2D {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    /// Build a new rectangle formed of the union of this rectangle and a point.
    ///
    /// The union is the smallest rectangle enclosing both the rectangle and the point. If the
    /// point is already inside the rectangle, this method returns a copy of the rectangle.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// let u = r.union_point(Vec2::new(3., 6.));
    /// assert!(u.min.abs_diff_eq(Vec2::ZERO, 1e-5));
    /// assert!(u.max.abs_diff_eq(Vec2::new(5., 6.), 1e-5));
    /// ```
    #[inline]
    pub fn union_point(&self, other: IVec2) -> Rect2D {
        Rect2D {
            min: self.min.min(other),
            max: self.max.max(other),
        }
    }

    /// Build a new rectangle formed of the intersection of this rectangle and another rectangle.
    ///
    /// The intersection is the largest rectangle enclosed in both rectangles. If the intersection
    /// is empty, this method returns an empty rectangle ([`Rect::is_empty()`] returns `true`), but
    /// the actual values of [`Rect::min`] and [`Rect::max`] are implementation-dependent.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r1 = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// let r2 = Rect::new(1., -1., 3., 3.); // w=2 h=4
    /// let r = r1.intersect(r2);
    /// assert!(r.min.abs_diff_eq(Vec2::new(1., 0.), 1e-5));
    /// assert!(r.max.abs_diff_eq(Vec2::new(3., 1.), 1e-5));
    /// ```
    #[inline]
    pub fn intersect(&self, other: Rect2D) -> Rect2D {
        let mut r = Rect2D {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        };
        // Collapse min over max to enforce invariants and ensure e.g. width() or
        // height() never return a negative value.
        r.min = r.min.min(r.max);
        r
    }

    /// Create a new rectangle with a constant inset.
    ///
    /// The inset is the extra border on all sides. A positive inset produces a larger rectangle,
    /// while a negative inset is allowed and produces a smaller rectangle. If the inset is negative
    /// and its absolute value is larger than the rectangle half-size, the created rectangle is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bevy_math::{Rect, Vec2};
    /// let r = Rect::new(0., 0., 5., 1.); // w=5 h=1
    /// let r2 = r.inset(3.); // w=11 h=7
    /// assert!(r2.min.abs_diff_eq(Vec2::splat(-3.), 1e-5));
    /// assert!(r2.max.abs_diff_eq(Vec2::new(8., 4.), 1e-5));
    /// ```
    #[inline]
    pub fn inset(&self, inset: i32) -> Rect2D {
        let mut r = Rect2D {
            min: self.min - inset,
            max: self.max + inset,
        };
        // Collapse min over max to enforce invariants and ensure e.g. width() or
        // height() never return a negative value.
        r.min = r.min.min(r.max);
        r
    }

    #[inline]
    pub fn index_for_point(&self, point: IVec2) -> Option<usize> {
        if !self.contains_exclusive_max(point.as_vec2()) {
            return None;
        }
        let top_left = self.min;
        // Distance_y * size_x  + Distance_x
        //Some(((top_left.y + point.y) * self.size().x + (top_left.x + point.x)) as usize)
        Some(((-top_left.y + point.y) * self.size().x + (-top_left.x + point.x)) as usize)
    }
}
