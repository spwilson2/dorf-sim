pub mod camera_frame;
pub mod local_map;
pub mod pathing;

use self::camera_frame::*;
use self::local_map::*;
use self::pathing::*;
use crate::prelude::*;
use crate::terminal::render::CharTexture;

#[derive(Default)]
pub struct ScriptPlugin();

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        log::debug!("Initializing ScriptPlugin");
        app.add_startup_system(spawn_centerpoint);
        add_pathing_systems(app, false);
        add_local_map_systems(app, true);
        add_camera_frame_systems(app, true);
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
