pub mod camera;
pub mod input;
pub mod render;

pub mod display;

pub use camera::*;
pub use input::*;
pub use render::*;

use crate::prelude::*;

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
