pub mod pathing;

use std::{
    cmp::{max, min, Reverse},
    collections::{BinaryHeap, HashMap},
};

use crate::{
    prelude::*,
    script::pathing::{sys_assign_new_goal, CollisionGridCache, MAP_DIMMENSIONS},
    terminal::{
        camera::{CameraResized, TerminalCamera2d},
        render::CharTexture,
    },
};

use bevy::{input::keyboard::KeyboardInput, transform};
use bevy::{input::ButtonState, utils::Uuid};
use ordered_float::OrderedFloat;

#[derive(Default)]
pub struct ScriptPlugin();

fn add_pathing_systems(app: &mut App, enabled: bool) {
    if enabled {
        log::debug!("pathing system: enabled");
        app.insert_resource(CollisionGridCache::new(IVec2::default(), MAP_DIMMENSIONS))
            .add_system(pathing::system_move_on_optimal_path)
            .add_system(pathing::sys_update_collision_cache)
            .add_system(
                pathing::system_assign_optimal_path.after(pathing::sys_update_collision_cache),
            )
            .add_system(pathing::sys_handle_collisions)
            .add_startup_system(pathing::spawn_collider_walls)
            .add_system(pathing::spawn_mv_player_over_time);
    } else {
        log::debug!("pathing system: disabled");
    }
}
impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        log::debug!("Initializing ScriptPlugin");
        app.add_system(handle_camera_movement_keys)
            .add_system(sys_assign_new_goal)
            .add_system(handle_camera_resized)
            .add_startup_system(spawn_textures);
        add_pathing_systems(app, false);
    }
}
/// - Create a player with a random intended path
///    - new entity:
///         - new CurLocation
///         - new GoalLocation
/// - Get the actual steps to take to get to that path using a*
///    - ref CurLocation
///    - del GoalLocation
///    - new TargetLocation
/// - Move the player to that position along the routed path
///    - mut CurLocation
///    -
/// - Player at destitation, then give them a new random path

fn spawn_textures(mut cmd: Commands) {
    let loc = Vec3::splat(1000.0);
    let vert_wall = CharTexture { texture: '-' };
    let vert_wall_trans = Transform2D {
        scale: UVec2::splat(1),
        loc,
    };
    let side_wall = CharTexture { texture: '|' };
    let side_wall_trans = Transform2D {
        scale: UVec2::splat(1),
        loc,
    };
    cmd.spawn((
        CharTexture { texture: '0' },
        Transform2D {
            scale: UVec2::splat(1),
            loc: Vec3::new(0.0, 0.0, 1.0),
        },
    ));
    cmd.spawn((
        CharTexture { texture: '2' },
        Transform2D {
            scale: UVec2::splat(1),
            loc: Vec3::new(2.0, 0.0, 1.0),
        },
    ));
    cmd.spawn((
        CharTexture { texture: '1' },
        Transform2D {
            scale: UVec2::splat(1),
            loc: Vec3::new(1.0, 0.0, 1.0),
        },
    ));
    cmd.spawn_batch([
        CameraFrameWallBundle {
            texture: side_wall.clone(),
            transform: side_wall_trans.clone(),
            side: CameraSide::Right,
        },
        CameraFrameWallBundle {
            texture: side_wall,
            transform: side_wall_trans.clone(),
            side: CameraSide::Left,
        },
        CameraFrameWallBundle {
            texture: vert_wall.clone(),
            transform: vert_wall_trans,
            side: CameraSide::Top,
        },
        CameraFrameWallBundle {
            texture: vert_wall,
            transform: side_wall_trans,
            side: CameraSide::Bottom,
        },
    ]);
}

/// Marker component type to indicate the CameraFrame Entity.
#[derive(Component, Default)]
struct CameraFrame {}

#[derive(Bundle)]
struct CameraFrameWallBundle {
    texture: CharTexture,
    transform: Transform2D,
    side: CameraSide,
}

#[derive(Component)]
enum CameraSide {
    Left,
    Right,
    Top,
    Bottom,
}

fn handle_camera_resized(
    mut walls: Query<(&mut Transform2D, &CameraSide)>,
    camera: Res<TerminalCamera2d>,
    mut event: EventReader<CameraResized>,
) {
    if let Some(event) = event.iter().last() {
        center_camera_frame(&camera, &mut walls)
    }
}

fn center_camera_frame(
    camera: &TerminalCamera2d,
    walls: &mut Query<(&mut Transform2D, &CameraSide)>,
) {
    for mut wall in walls.iter_mut() {
        match *wall.1 {
            CameraSide::Left => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.scale.x = 1;
                wall.0.scale.y = camera.dim().y;
            }
            CameraSide::Right => {
                wall.0.loc.x = camera.dim().x as f32 + camera.loc().x - 1.0;
                wall.0.loc.y = camera.loc().y;
                wall.0.scale.x = 1;
                wall.0.scale.y = camera.dim().y;
            }
            CameraSide::Top => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.scale.x = camera.dim().x;
                wall.0.scale.y = 1;
            }
            CameraSide::Bottom => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = camera.dim().y as f32 + camera.loc().y - 1.0;
                wall.0.scale.x = camera.dim().x;
                wall.0.scale.y = 1;
            }
        }
    }
}

fn move_camera(
    direction: Vec2,
    camera: &mut ResMut<TerminalCamera2d>,
    walls: &mut Query<(&mut Transform2D, &CameraSide)>,
) {
    *camera.loc_mut() += Vec3::new(direction.x, direction.y, 0.0);
    center_camera_frame(&*camera, walls);
}

fn handle_camera_movement_keys(
    mut input: EventReader<KeyboardInput>,
    mut camera: ResMut<TerminalCamera2d>,
    mut walls: Query<(&mut Transform2D, &CameraSide)>,
) {
    for e in input.iter() {
        if e.state != ButtonState::Pressed {
            continue;
        }
        if let Some(k) = e.key_code {
            match k {
                KeyCode::D => move_camera(Vec2::new(1.0, 0.0), &mut camera, &mut walls),
                KeyCode::A => move_camera(Vec2::new(-1.0, 0.0), &mut camera, &mut walls),
                KeyCode::W => move_camera(Vec2::new(0.0, -1.0), &mut camera, &mut walls),
                KeyCode::S => move_camera(Vec2::new(0.0, 1.0), &mut camera, &mut walls),
                _ => (),
            }
        }
    }
}
