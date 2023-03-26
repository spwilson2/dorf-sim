use std::io::stdout;

use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, size, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};

use crate::prelude::*;
use crate::onexit::RegisterOnExit;

#[derive(Default)]
pub struct TerminalDisplayPlugin {}


impl Plugin for TerminalDisplayPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(crate::onexit::OnExitPlugin{})
        .add_startup_system(init);
    }
}

fn init(mut onexit_register: EventWriter<RegisterOnExit>) {
    let (cols, rows) = size().unwrap();
    enable_raw_mode().unwrap();
    // Resize terminal and scroll up.
    execute!(
        stdout(),
        EnterAlternateScreen,
    ).unwrap();
    onexit_register.send(RegisterOnExit(cleanup));
}

fn cleanup() {
    log::info!("Performing terminal cleanup");
    disable_raw_mode().unwrap();
    execute!(
        stdout(),
        LeaveAlternateScreen,
    ).unwrap();
}
