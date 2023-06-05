pub mod camera;
pub mod char_mesh;
pub mod char_texture;
pub mod display;
pub mod input;
pub mod render;

use crate::prelude::*;
pub use camera::*;
pub use char_mesh::*;
pub use char_texture::*;
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
