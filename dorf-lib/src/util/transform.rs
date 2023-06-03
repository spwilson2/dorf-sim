use bevy::a11y::accesskit::Rect;

use crate::prelude::*;

#[derive(Default, Debug, Component, Clone)]
pub struct Transform2D {
    pub scale: UVec2,
    // Movement will be partial, so we need loc to be flaot
    pub loc: Vec3,
}

impl Transform2D {
    pub fn z_lvl(&self) -> i32 {
        self.loc.z.round() as i32
    }

    pub fn as_tile(&self) -> IVec2 {
        tile_from_vec2(self.loc.xy())
    }

    pub fn as_rect2d(&self) -> Rect2D {
        Rect2D::from_transform2d(self)
    }
}
