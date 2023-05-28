use crate::{
    prelude::*,
    terminal::{
        camera::{CameraResized, TerminalCamera2d},
        render::CharTexture,
        render::Transform,
    },
};

use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;

#[derive(Default)]
pub struct ScriptPlugin();

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_camera_movement_keys)
            .add_system(handle_camera_resized)
            .add_system(system_assign_optimal_path)
            .add_system(system_move_on_optimal_path)
            .add_startup_system(spawn_mv_player)
            .add_startup_system(spawn_textures);
    }
}

#[derive(Component)]
struct MovePath {
    steps: Vec<Vec2>,
}

#[derive(Component)]
struct GoalLoc(Vec2);
#[derive(Component)]
struct Speed(f32);

#[derive(Bundle)]
struct Player {
    speed: Speed,
    goal: GoalLoc,
    rect: CharTexture,
    transform: Transform,
}

fn system_assign_optimal_path(mut cmd: Commands, q: Query<(Entity, &GoalLoc)>) {
    for (entity, goal) in q.iter() {
        // TODO: Calculate optimal path, for now will work since nothing to
        // colldie with. Will need to figure out how to do efficient collision
        // detec later.
        cmd.entity(entity)
            .insert(MovePath {
                steps: vec![goal.0],
            })
            .remove::<GoalLoc>();
    }
}

fn system_move_on_optimal_path(
    mut cmd: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut MovePath, &mut Transform, &Speed)>,
) {
    log::info!("in move optimal");
    for (entity, mut path, mut rect, speed) in q.iter_mut() {
        let mut travel = speed.0 * time.delta().as_secs_f32();
        loop {
            // While we have time to travel, contiue doing so.

            // First check if there's nothing left to move
            if path.steps.is_empty() {
                cmd.entity(entity).remove::<MovePath>();
                // TODO: Add another goal path
                break;
            }

            let dist = rect.loc.distance(*path.steps.last().unwrap());
            if dist <= travel {
                travel -= dist;
                // We've move to the point, need to continue to the next point.
                rect.loc = path.steps.pop().unwrap();
            } else {
                // We can't move directly to the point, let's get as close as we can
                let direction = (*path.steps.last().unwrap() - rect.loc).normalize();
                rect.loc += direction * travel;
                break;
            }
        }
    }
}

fn spawn_mv_player(mut cmd: Commands) {
    cmd.spawn(Player {
        speed: Speed(1.5),
        goal: GoalLoc(Vec2::new(10.0, 10.0)),
        rect: CharTexture { texture: 'p' },
        transform: Transform {
            size: Vec2::new(1.0, 1.0),
            loc: Vec2::new(0.0, 0.0),
            z_lvl: 2,
        },
    });
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
    let vert_wall = CharTexture { texture: '-' };
    let vert_wall_trans = Transform {
        size: Vec2::new(1.0, 2.0),
        loc: Vec2::new(0.0, 0.0),
        z_lvl: 1000,
    };
    let side_wall = CharTexture { texture: '|' };
    let side_wall_trans = Transform {
        size: Vec2::new(2.0, 1.0),
        loc: Vec2::new(0.0, 0.0),
        z_lvl: 1000,
    };
    cmd.spawn((
        CharTexture { texture: 'a' },
        Transform {
            size: Vec2::new(1.0, 1.0),
            loc: Vec2::new(0.0, 0.0),
            z_lvl: 1,
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
    transform: Transform,
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
    mut walls: Query<(&mut Transform, &CameraSide)>,
    camera: Res<TerminalCamera2d>,
    mut event: EventReader<CameraResized>,
) {
    if let Some(event) = event.iter().last() {
        center_camera_frame(&camera, &mut walls)
    }
}

fn center_camera_frame(
    camera: &TerminalCamera2d,
    walls: &mut Query<(&mut Transform, &CameraSide)>,
) {
    for mut wall in walls.iter_mut() {
        match *wall.1 {
            CameraSide::Left => {
                wall.0.loc.x = -camera.dim().x / 2.0 + 1.0 + camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.size.x = 1.0;
                wall.0.size.y = camera.dim().y;
            }
            CameraSide::Right => {
                wall.0.loc.x = camera.dim().x / 2.0 + camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.size.x = 1.0;
                wall.0.size.y = camera.dim().y;
            }
            CameraSide::Top => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = -camera.dim().y / 2.0 + 1.0 + camera.loc().y;
                wall.0.size.x = camera.dim().x;
                wall.0.size.y = 1.0;
            }
            CameraSide::Bottom => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = camera.dim().y / 2.0 + camera.loc().y;
                wall.0.size.x = camera.dim().x;
                wall.0.size.y = 1.0;
            }
        }
    }
}

fn move_camera(
    direction: Vec2,
    camera: &mut ResMut<TerminalCamera2d>,
    walls: &mut Query<(&mut Transform, &CameraSide)>,
) {
    camera.move_by(Vec3::new(direction.x, direction.y, 0.0));
    center_camera_frame(&*camera, walls);
}

fn handle_camera_movement_keys(
    mut input: EventReader<KeyboardInput>,
    mut camera: ResMut<TerminalCamera2d>,
    mut walls: Query<(&mut Transform, &CameraSide)>,
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
