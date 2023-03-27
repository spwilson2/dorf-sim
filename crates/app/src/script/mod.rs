use crate::{
    prelude::*,
    terminal::{
        camera::{CameraResized, TerminalCamera2d},
        render::TextureRect,
    },
};

use bevy::input::keyboard::KeyboardInput;

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
        dim: Vec2::new(2.0, 2.0),
        loc: Vec2::new(0.0, 0.0),
        loc_z: 1.0,
    });
    cmd.spawn(TextureRect {
        texture: 'b',
        dim: Vec2::new(1.0, 1.0),
        loc: Vec2::new(0.0, 0.0),
        loc_z: 2.0,
    });
    cmd.spawn(TextureRect {
        texture: '.',
        dim: Vec2::new(1000.0, 1000.0),
        loc: Vec2::new(0.0, 0.0),
        loc_z: 1.0,
    });

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
    mut event: EventReader<CameraResized>,
) {
    if let Some(event) = event.iter().last() {
        for mut wall in walls.iter_mut() {
            match *wall.1 {
                CameraSide::Left => {
                    wall.0.loc.x = (-(event.0.x as isize / 2) + 1) as f32;
                    wall.0.loc.y = 0.0;
                    wall.0.dim.x = 1.0;
                    wall.0.dim.y = event.0.y;
                }
                CameraSide::Right => {
                    wall.0.loc.x = ((event.0.x as isize / 2) - 1) as f32;
                    wall.0.loc.y = 0.0;
                    wall.0.dim.x = 1.0;
                    wall.0.dim.y = event.0.y;
                }
                CameraSide::Top => {
                    wall.0.loc.y = (-(event.0.y as isize / 2) + 1) as f32;
                    wall.0.loc.x = 0.0;
                    wall.0.dim.x = event.0.x;
                    wall.0.dim.y = 1.0;
                }
                CameraSide::Bottom => {
                    wall.0.loc.y = ((event.0.y as isize / 2) - 1) as f32;
                    wall.0.loc.x = 0.0;
                    wall.0.dim.x = event.0.x;
                    wall.0.dim.y = 1.0;
                }
                //CameraSide::Right => todo!(),
                //CameraSide::Top => todo!(),
                //CameraSide::Bottom => todo!(),
            }
        }
    }
}

fn move_camera(
    direction: Vec2,
    mut camera: &mut ResMut<TerminalCamera2d>,
    mut walls: &mut Query<(&mut TextureRect, &CameraSide)>,
) {
    if direction.x != 0.0 {
        camera.move_x(direction.x);
        for mut wall in walls.iter_mut() {
            wall.0.loc.x += direction.x;
        }
    } else if direction.y != 0.0 {
        camera.move_y(direction.y);
        for mut wall in walls.iter_mut() {
            wall.0.loc.y += direction.y;
        }
    }
}

fn handle_camera_movement_keys(
    mut input: EventReader<KeyboardInput>,
    mut camera: ResMut<TerminalCamera2d>,
    mut walls: Query<(&mut TextureRect, &CameraSide)>,
) {
    for e in input.iter() {
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
