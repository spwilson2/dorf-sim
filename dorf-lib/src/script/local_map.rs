use crate::prelude::*;

pub const LOCAL_MAP_DIMMENSIONS: UVec2 = UVec2 { x: 100, y: 50 };

pub fn add_local_map_systems(app: &mut App, enabled: bool) {}

// TODO:

enum Biome {
    Forest,
    Ocean,
    Sand,
    Mountain,
}
