pub mod camera_frame;
pub mod local_map;
pub mod pathing;

use crate::terminal::*;
use bevy::app::AppExit;

use self::camera_frame::*;
use self::local_map::*;
use self::pathing::*;
use crate::prelude::*;

#[derive(Default)]
pub struct ScriptPlugin();

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        log::debug!("Initializing ScriptPlugin");
        app.add_startup_system(spawn_centerpoint)
            .add_system(sys_exit_key_handler);
        add_pathing_systems(app, true);
        add_local_map_systems(app, true);
        add_camera_frame_systems(app, true);
    }
}

fn sys_exit_key_handler(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.iter() {
        if let Some(k) = e.key_code {
            if [KeyCode::Escape, KeyCode::Q].contains(&k) {
                writer.send(AppExit);
            }
        }
    }
}

fn spawn_centerpoint(mut cmd: Commands) {
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
}
