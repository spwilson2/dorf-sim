use crate::prelude::*;
use crate::terminal::render::{TerminalCamera2d, TextureRect};

#[derive(Default)]
pub struct ScriptPlugin();

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(initialize_camera)
            .add_startup_system(spawn_textures);
    }
}

fn initialize_camera(mut camera: ResMut<TerminalCamera2d>) {
    camera.dim = Vec2::new(10.0, 10.0);
    camera.loc = Vec3::new(5.0, 5.0, 1.0);
}

fn spawn_textures(mut cmd: Commands) {
    cmd.spawn(TextureRect {
        texture: 'a',
        dim: Vec2::new(1.0, 1.0),
        loc: Vec2::new(5.0, 5.0),
        loc_z: 1.0,
    });
    cmd.spawn(TextureRect {
        texture: 'b',
        dim: Vec2::new(0.5, 0.5),
        loc: Vec2::new(5.0, 5.0),
        loc_z: 2.0,
    });
}
