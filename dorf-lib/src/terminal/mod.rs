pub mod camera;
pub mod input;
pub mod render;

pub mod display;

use crate::prelude::*;
pub use camera::*;
pub use input::*;
pub use render::*;

#[derive(Default)]
pub struct TerminalPlugin {}

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        log::info!("Initializing TerminalPlugin");
        app.add_plugin(self::input::TerminalInputPlugin::default())
            .add_plugin(self::display::TerminalDisplayPlugin::default())
            .add_plugin(self::render::TerminalRenderPlugin::default())
            .add_plugin(self::camera::TerminalCamera2dPlugin::default());
    }
}

#[derive(Bundle, Clone)]
pub struct CharTextureTransform {
    texture: CharTexture,
    transform: Transform2D,
}

/// Simple texture on top of transform
#[derive(Component, Debug, Clone)]
pub struct CharTexture {
    pub texture: char,
}

#[derive(Component)]
pub struct CharPaintGrid {
    grid: Grid2D<char>,
}
