use std::cmp::max;

use crate::{prelude::*, terminal::render::CharTexture};

pub const LOCAL_MAP_DIMMENSIONS: UVec2 = UVec2 { x: 100, y: 50 };

/// Number of points to seed with
pub const INIT_RATIO: f32 = 0.05;

pub fn add_local_map_systems(app: &mut App, enabled: bool) {}

// TODO:
pub fn spawn_random_global_map() {
    let grid: Grid2D<Option<Biome>> = Grid2D::new(IVec2::ZERO, LOCAL_MAP_DIMMENSIONS, None);

    let num_init_nodes = max(
        1,
        (INIT_RATIO * (LOCAL_MAP_DIMMENSIONS.x * LOCAL_MAP_DIMMENSIONS.y) as f32) as usize,
    );

    for i in 0..num_init_nodes {
        let x = fastrand::i32(0..LOCAL_MAP_DIMMENSIONS.x as i32);
        let y = fastrand::i32(0..LOCAL_MAP_DIMMENSIONS.y as i32);

        let biome: Option<Biome> = fastrand::u8(0..Biome::_Max as u8).try_into().ok();

        grid.set(IVec2 { x, y }, biome);
    }
    // TODO: constraint based generation:

    // - Start out by picking a couple of points to begin constraint solving for
    // - Iteratively begin placing nodes that meet constraints
}

#[derive(Component, Debug, Copy, Clone)]
#[repr(u8)]
enum Biome {
    Forest = 0,
    Ocean,
    Sand,
    Mountain,
    _Max,
}

#[derive(Bundle, Debug)]
struct BiomeBundle {
    biome: Biome,
    texture: CharTexture,
    transform: Transform2D,
}

impl Biome {
    fn new_bundle(biome: Biome, location: Vec2) -> BiomeBundle {
        let mut transform = Transform2D {
            scale: UVec2::splat(1),
            loc: location.xyy(),
        };
        transform.loc.z = 0.0;
        BiomeBundle {
            texture: biome.texture(),
            biome,
            transform,
        }
    }

    fn texture(&self) -> CharTexture {
        CharTexture {
            texture: match self {
                Biome::Forest => '|',
                Biome::Ocean => '~',
                Biome::Sand => '.',
                Biome::Mountain => '^',
            },
        }
    }
}
