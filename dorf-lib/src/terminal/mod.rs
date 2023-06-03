pub mod camera;
pub mod char_mesh;
pub mod display;
pub mod input;
pub mod render;

use crate::prelude::*;
pub use camera::*;
pub use char_mesh::*;
pub use input::*;
pub use render::*;

use self::display::DisplayBuffer;

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
#[derive(Component, Debug, Clone, PartialEq)]
pub struct CharTexture {
    pub c: char,
    pub rgb: Option<RGB>,
}
impl Default for CharTexture {
    fn default() -> Self {
        Self { c: ' ', rgb: None }
    }
}

impl CharTexture {
    pub fn new(texture: char, rgb: RGB) -> Self {
        Self {
            c: texture,
            rgb: Some(rgb),
        }
    }
    pub fn from_char(texture: char) -> Self {
        Self {
            c: texture,
            rgb: None,
        }
    }
}
