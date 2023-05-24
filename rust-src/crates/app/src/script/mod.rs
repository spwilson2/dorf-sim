use crate::{
    prelude::*,
    terminal::{
        camera::{CameraResized, TerminalCamera2d},
        render::TextureRect,
    },
};

use bevy::input::keyboard::{ButtonState, KeyboardInput};

#[derive(Default)]
pub struct ScriptPlugin();

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_camera_movement_keys)
            .add_system(handle_camera_resized)
            .add_startup_system(spawn_textures);
    }
}

fn spawn_textures(mut cmd: Commands) {
    cmd.spawn(TextureRect {
        texture: 'a',
        dim: Vec2::new(2.0, 1.0),
        loc: Vec2::new(0.0, 0.0),
        loc_z: 1.0,
    });
    //cmd.spawn(TextureRect {
    //    texture: 'b',
    //    dim: Vec2::new(1.0, 1.0),
    //    loc: Vec2::new(0.0, 0.0),
    //    loc_z: 2.0,
    //});
    //cmd.spawn(TextureRect {
    //    texture: '.',
    //    dim: Vec2::new(1000.0, 1000.0),
    //    loc: Vec2::new(0.0, 0.0),
    //    loc_z: 1.0,
    //});

    let vert_wall = TextureRect {
        texture: '-',
        dim: Vec2::new(1.0, 2.0),
        loc: Vec2::new(0.0, 0.0),
        loc_z: 1000.0,
    };
    let side_wall = TextureRect {
        texture: '|',
        dim: Vec2::new(2.0, 1.0),
        loc: Vec2::new(0.0, 0.0),
        loc_z: 1000.0,
    };
    cmd.spawn_batch([
        CameraFrameWallBundle {
            texture: side_wall.clone(),
            side: CameraSide::Right,
        },
        CameraFrameWallBundle {
            texture: side_wall,
            side: CameraSide::Left,
        },
        CameraFrameWallBundle {
            texture: vert_wall.clone(),
            side: CameraSide::Top,
        },
        CameraFrameWallBundle {
            texture: vert_wall,
            side: CameraSide::Bottom,
        },
    ]);
}

/// Marker component type to indicate the CameraFrame Entity.
#[derive(Component, Default)]
struct CameraFrame {}

#[derive(Bundle)]
struct CameraFrameWallBundle {
    texture: TextureRect,
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
    mut walls: Query<(&mut TextureRect, &CameraSide)>,
    camera: Res<TerminalCamera2d>,
    mut event: EventReader<CameraResized>,
) {
    if let Some(event) = event.iter().last() {
        center_camera_frame(&camera, &mut walls)
    }
}

fn center_camera_frame(
    camera: &TerminalCamera2d,
    walls: &mut Query<(&mut TextureRect, &CameraSide)>,
) {
    for mut wall in walls.iter_mut() {
        match *wall.1 {
            CameraSide::Left => {
                wall.0.loc.x = -camera.dim().x / 2.0 + 1.0 + camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.dim.x = 1.0;
                wall.0.dim.y = camera.dim().y;
            }
            CameraSide::Right => {
                wall.0.loc.x = camera.dim().x / 2.0 + camera.loc().x;
                wall.0.loc.y = camera.loc().y;
                wall.0.dim.x = 1.0;
                wall.0.dim.y = camera.dim().y;
            }
            CameraSide::Top => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = -camera.dim().y / 2.0 + 1.0 + camera.loc().y;
                wall.0.dim.x = camera.dim().x;
                wall.0.dim.y = 1.0;
            }
            CameraSide::Bottom => {
                wall.0.loc.x = camera.loc().x;
                wall.0.loc.y = camera.dim().y / 2.0 + camera.loc().y;
                wall.0.dim.x = camera.dim().x;
                wall.0.dim.y = 1.0;
            }
        }
    }
}

fn move_camera(
    direction: Vec2,
    camera: &mut ResMut<TerminalCamera2d>,
    walls: &mut Query<(&mut TextureRect, &CameraSide)>,
) {
    camera.move_by(Vec3::new(direction.x, direction.y, 0.0));
    center_camera_frame(&*camera, walls);
}

fn handle_camera_movement_keys(
    mut input: EventReader<KeyboardInput>,
    mut camera: ResMut<TerminalCamera2d>,
    mut walls: Query<(&mut TextureRect, &CameraSide)>,
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
