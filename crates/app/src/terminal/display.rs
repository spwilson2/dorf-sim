use std::io::stdout;

use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, size, enable_raw_mode, EnterAlternateScreen, SetSize, ScrollUp, LeaveAlternateScreen};

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
        SetSize(10, 10),
        ScrollUp(5),
        LeaveAlternateScreen,
    ).unwrap();
    onexit_register.send(RegisterOnExit(cleanup));
}

fn cleanup() {
    disable_raw_mode().unwrap()
}
