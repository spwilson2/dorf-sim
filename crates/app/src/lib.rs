//! This example will display a simple menu using Bevy UI where you can start a new game,
//! change some settings or quit. There is no actual game, it will just display the current
//! settings for 5 seconds before going back to the menu.

use bevy::prelude::*;

mod scene;

pub fn app_main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Insert as resource the initial value for the settings resources
        .insert_resource(scene::DisplayQuality::Medium)
        .insert_resource(scene::Volume(7))
        .add_startup_system(setup)
        // Declare the game state, whose starting value is determined by the `Default` trait
        .add_state::<scene::GameState>()
        // Adds the plugins for each state
        .add_plugin(scene::splash::SplashPlugin)
        .add_plugin(scene::menu::MenuPlugin)
        .add_plugin(scene::game::GamePlugin)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}


