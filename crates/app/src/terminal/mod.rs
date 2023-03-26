mod input;
mod display;
pub mod render;

use crate::prelude::*;

#[derive(Default)]
pub struct TerminalPlugin {}

impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(self::input::TerminalInputPlugin::default())
        .add_plugin(self::display::TerminalDisplayPlugin::default())
        .add_plugin(self::render::TerminalRenderPlugin::default());
    }
}