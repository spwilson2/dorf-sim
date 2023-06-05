use std::cmp::max;

use bevy::tasks::{AsyncComputeTaskPool, Task};
use noise::{
    utils::{ImageRenderer, NoiseMapBuilder, PlaneMapBuilder},
    NoiseFn,
};

use crate::prelude::*;

pub const LOCAL_MAP_DIMMENSIONS: UVec2 = UVec2 { x: 100, y: 50 };
//
///// Number of points to seed with
//pub const INIT_RATIO: f32 = 0.05;
//
pub fn add_local_map_systems(app: &mut App, enabled: bool) {
    app.add_startup_system(sys_init_spawn_gen_req);
    //.add_startup_system(gen_map)
    //.add_system(sys_prepare_gen_map_task);
}

pub fn spawn_random_global_map() {
    //let grid: Grid2D<Option<Biome>> = Grid2D::new(IVec2::ZERO, LOCAL_MAP_DIMMENSIONS, None);

    //let num_init_nodes = max(
    //    1,
    //    (INIT_RATIO * (LOCAL_MAP_DIMMENSIONS.x * LOCAL_MAP_DIMMENSIONS.y) as f32) as usize,
    //);

    //for i in 0..num_init_nodes {
    //    let x = fastrand::i32(0..LOCAL_MAP_DIMMENSIONS.x as i32);
    //    let y = fastrand::i32(0..LOCAL_MAP_DIMMENSIONS.y as i32);

    //    let biome: Option<Biome> = fastrand::u8(0..Biome::_Max as u8).try_into().ok();

    //    grid.set(IVec2 { x, y }, biome);
    //}
    // TODO: constraint based generation:

    // - Start out by picking a couple of points to begin constraint solving for
    // - Iteratively begin placing nodes that meet constraints
}

#[derive(Component, Debug, Copy, Clone)]
enum Biome {
    //Hill,
    Forest,
    Grassland,
    //Desert,
    Beach,
    Ocean,
    Sand,
    Mountain,
}

#[derive(Bundle, Debug)]
struct BiomeBundle {
    biome: Biome,
    texture: CharTexture,
    transform: Transform2D,
}

impl BiomeBundle {
    fn new(biome: Biome, location: Vec2) -> BiomeBundle {
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
}
impl Biome {
    fn texture(&self) -> CharTexture {
        CharTexture {
            c: match self {
                Biome::Forest => '|',
                Biome::Ocean => '~',
                Biome::Sand => '.',
                Biome::Mountain => '^',
                Biome::Grassland => todo!(),
                Biome::Beach => todo!(),
                //Biome::Desert => todo!(),
                //Biome::Hill => todo!(),
            },
            rgb: default(),
        }
    }
}
struct MapGenResult {}

#[derive(Component)]
struct MapGenTask {
    task: Task<MapGenResult>,
}

#[derive(Component)]
struct MapGenTaskRequest {}

// Testing start code to kick off generating the map.
pub fn sys_init_spawn_gen_req(cnt: Local<usize>, mut cmds: Commands) {
    cmds.spawn(MapGenTaskRequest {});
}

// When we spawn the map, we need to do it asynchronously so as not to block the main thread.
fn sys_prepare_gen_map_task(req_q: Query<(Entity, &MapGenTaskRequest)>, mut cmds: Commands) {
    for (entity, req) in req_q.iter() {
        let pool = AsyncComputeTaskPool::get();
        let task = pool.spawn(async move {
            gen_map();
            MapGenResult {}
        });

        cmds.entity(entity).insert(MapGenTask { task });
        cmds.entity(entity).remove::<MapGenTaskRequest>();
    }
}

// TODO: Leverage this code to generate the map.
fn gen_map() {
    log::info!("Generating map!");
    use noise::{utils::*, *};
    // Base wood texture. Uses concentric cylinders aligned on the z-axis, like a log.
    let base_wood = Cylinders::new().set_frequency(16.0);
    render_noise(&base_wood, "0_base_wood.png");

    // Basic Multifractal noise to use for the wood grain.
    let wood_grain_noise = BasicMulti::<Perlin>::new(0)
        .set_frequency(48.0)
        .set_persistence(0.5)
        .set_lacunarity(2.20703125)
        .set_octaves(3);
    render_noise(&wood_grain_noise, "1_grain_noise.png");

    // Stretch the perlin noise in the same direction as the center of the log. Should
    // produce a nice wood-grain texture.
    let scaled_base_wood_grain = ScalePoint::new(wood_grain_noise).set_z_scale(0.25);
    render_noise(&scaled_base_wood_grain, "2_scaled_grain_noise.png");

    // Scale the wood-grain values so that they can be added to the base wood texture.
    let wood_grain = ScaleBias::new(scaled_base_wood_grain)
        .set_scale(0.25)
        .set_bias(0.125);
    render_noise(&wood_grain, "3_biased_wood_grain.png");

    // Add the wood grain texture to the base wood texture.
    let combined_wood = Add::new(base_wood, wood_grain);
    render_noise(&combined_wood, "4_combined.png");

    // Slightly perturb the wood to create a more realistic texture.
    let perturbed_wood = Turbulence::<_, Perlin>::new(combined_wood)
        .set_seed(1)
        .set_frequency(4.0)
        .set_power(1.0 / 256.0)
        .set_roughness(4);
    render_noise(&perturbed_wood, "5_pertubed.png");

    // Cut the wood texture a small distance from the center of the log.
    let translated_wood = TranslatePoint::new(perturbed_wood).set_y_translation(1.48);

    // Set the cut on a angle to produce a more interesting texture.
    let rotated_wood = RotatePoint::new(translated_wood).set_angles(84.0, 0.0, 0.0, 0.0);

    // Finally, perturb the wood texture again to produce the final texture.
    let final_wood = Turbulence::<_, Perlin>::new(rotated_wood)
        .set_seed(2)
        .set_frequency(2.0)
        .set_power(1.0 / 64.0)
        .set_roughness(4);

    let planar_texture = PlaneMapBuilder::<_, 2>::new(final_wood)
        .set_size(1024, 1024)
        .build();
}

fn render_noise<SourceModule>(src: &SourceModule, string: &str)
where
    SourceModule: NoiseFn<f64, 2>,
{
    let planar_texture = PlaneMapBuilder::<_, 2>::new(src.clone())
        .set_size(1024, 1024)
        .build();
    let mut renderer = ImageRenderer::new(); //.set_gradient(wood_gradient);
    renderer.render(&planar_texture).write_to_file(string);
}
