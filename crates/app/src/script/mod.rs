use crate::{
    prelude::*,
    terminal::{camera::TerminalCamera2d, render::TextureRect},
};

#[derive(Default)]
pub struct ScriptPlugin();

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_textures);
    }
}

fn spawn_textures(mut cmd: Commands) {
    cmd.spawn(TextureRect {
        texture: 'a',
        dim: Vec2::new(2.0, 2.0),
        loc: Vec2::new(5.0, 5.0),
        loc_z: 1.0,
    });
    cmd.spawn(TextureRect {
        texture: 'b',
        dim: Vec2::new(1.0, 1.0),
        loc: Vec2::new(5.0, 5.0),
        loc_z: 2.0,
    });
}
