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
}
