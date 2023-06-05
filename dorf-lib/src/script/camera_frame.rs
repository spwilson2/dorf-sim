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
    let vert_wall = CharTexture::from_char('-');
    let vert_wall_trans = Transform2D {
        scale: UVec2::new(0, 1),
        loc,
    };
    let side_wall = CharTexture::from_char('|');
    let side_wall_trans = Transform2D {
        scale: UVec2::new(1, 0),
        loc,
    };
    cmd.spawn_batch([
        CameraFrameWallBundle {
            texture: side_wall.clone(),
            transform: side_wall_trans.clone(),
            side: CameraSide::Right,
            ui_component: UIComponent {
                local_pos: Vec3::new(1.0, 0.0, 256.0),
                relative_pos: true,
            },
        },
        CameraFrameWallBundle {
            texture: side_wall,
            transform: side_wall_trans.clone(),
            side: CameraSide::Left,
            ui_component: UIComponent {
                local_pos: Vec3::new(0.0, 0.0, 256.0),
                relative_pos: true,
            },
        },
        CameraFrameWallBundle {
            texture: vert_wall.clone(),
            transform: vert_wall_trans.clone(),
            side: CameraSide::Top,
            ui_component: UIComponent {
                local_pos: Vec3::new(0.0, 0.0, 256.0),
                relative_pos: true,
            },
        },
        CameraFrameWallBundle {
            texture: vert_wall,
            transform: vert_wall_trans.clone(),
            side: CameraSide::Bottom,
            ui_component: UIComponent {
                local_pos: Vec3::new(0.0, 1.0, 256.0),
                relative_pos: true,
            },
        },
    ]);
}

#[derive(Bundle)]
struct CameraFrameWallBundle {
    texture: CharTexture,
    transform: Transform2D,
    side: CameraSide,
    ui_component: UIComponent,
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
    // TODO: Probbaly shouldn't use an event for this and should just monitor the camera's transform..?
    mut event: EventReader<CameraResized>,
) {
    if let Some(event) = event.iter().last() {
        resize_walls(&camera.dim(), &mut walls)
    }
}
fn resize_walls(cam_size: &UVec2, walls: &mut Query<(&mut Transform2D, &CameraSide)>) {
    for mut wall in walls.iter_mut() {
        match *wall.1 {
            CameraSide::Left | CameraSide::Right => {
                wall.0.scale.y = cam_size.y;
            }
            CameraSide::Top | CameraSide::Bottom => {
                wall.0.scale.x = cam_size.x;
            }
        }
    }
}

fn move_camera(direction: Vec2, camera: &mut ResMut<TerminalCamera2D>) {
    *camera.loc_mut() += Vec3::new(direction.x, direction.y, 0.0);
}

fn handle_camera_movement_keys(
    mut input: EventReader<KeyboardInput>,
    mut camera: ResMut<TerminalCamera2D>,
) {
    for e in input.iter() {
        if e.state != ButtonState::Pressed {
            continue;
        }
        if let Some(k) = e.key_code {
            match k {
                KeyCode::D => move_camera(Vec2::new(1.0, 0.0), &mut camera),
                KeyCode::A => move_camera(Vec2::new(-1.0, 0.0), &mut camera),
                KeyCode::W => move_camera(Vec2::new(0.0, -1.0), &mut camera),
                KeyCode::S => move_camera(Vec2::new(0.0, 1.0), &mut camera),
                _ => (),
            }
        }
    }
}
