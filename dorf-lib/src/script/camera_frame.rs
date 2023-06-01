use crate::prelude::*;
use bevy::{input::keyboard::KeyboardInput, transform};
use bevy::{input::ButtonState, utils::Uuid};

pub fn add_camera_frame_systems(app: &mut App, enabled: bool) {
    app.add_system(handle_camera_movement_keys)
        .add_system(handle_camera_resized)
        .add_startup_system(spawn_camera_frame);
}

fn spawn_camera_frame(mut cmd: Commands) {
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
    camera: Res<TerminalCamera2D>,
    mut event: EventReader<CameraResized>,
) {
    if let Some(event) = event.iter().last() {
        center_camera_frame(&camera, &mut walls)
    }
}

fn center_camera_frame(
    camera: &TerminalCamera2D,
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
    camera: &mut ResMut<TerminalCamera2D>,
    walls: &mut Query<(&mut Transform2D, &CameraSide)>,
) {
    *camera.loc_mut() += Vec3::new(direction.x, direction.y, 0.0);
    center_camera_frame(&*camera, walls);
}

fn handle_camera_movement_keys(
    mut input: EventReader<KeyboardInput>,
    mut camera: ResMut<TerminalCamera2D>,
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
