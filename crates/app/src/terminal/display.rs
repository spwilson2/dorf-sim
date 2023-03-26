use std::io::{stdout, Write};

use crossterm::cursor::MoveTo;
use crossterm::{execute, QueueableCommand};
use crossterm::terminal::{disable_raw_mode, size, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, BeginSynchronizedUpdate, Clear, EndSynchronizedUpdate, ClearType};
use crossterm::queue;


use crate::prelude::*;
use crate::onexit::RegisterOnExit;

#[derive(Default)]
pub struct TerminalDisplayPlugin {}


impl Plugin for TerminalDisplayPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugin(crate::onexit::OnExitPlugin{})
        .add_startup_system(init)
        .insert_resource(TerminalDisplayBuffer::init_from_screen())
        .add_system(paint);
    }
}

fn init(mut onexit_register: EventWriter<RegisterOnExit>) {
    enable_raw_mode().unwrap();
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

fn paint(term_buffer: Res<TerminalDisplayBuffer>) {
    // Detect if there's an update. 
    // If so, perform the render. (TODO: Maybe only render part if necessary?)
    if term_buffer.is_changed() {
        let mut stdout = stdout().lock();
        queue!(stdout, BeginSynchronizedUpdate, Clear(ClearType::All), MoveTo(0,0)).unwrap();


        //for (i, c) in term_buffer.0.buf.iter().enumerate() {
        //    let x = i / term_buffer.0.width as usize;
        //    let y = i % term_buffer.0.height as usize;
        //    // TODO: validate the display will still render for given coords, otherwise log a warning and try our best with truncation.
        //}
        //
        // For now, let's just check if  the dimmensions look like they're gonna be fkd and log a warning, we can updated/fix in the next pass.
        if cfg!(debug_assertions) {
            let (width, height) = get_term_size();
            if (width, height) != (term_buffer.0.width, term_buffer.0.height) {
                log::warn!("Write buffer size: {:?} doesn't match current terminal size: {:?}", (term_buffer.0.width, term_buffer.0.height), (width, height));
            }

        }

        // The buffer should match its internal dimmensions
        debug_assert_eq!(term_buffer.0.buf.len(), term_buffer.0.width as usize * term_buffer.0.height as usize);

        // Full pass re-render...
        stdout.write(term_buffer.0.buf.iter().map(|c| *c as u8).collect::<Vec<u8>>().as_slice()).unwrap();

        stdout.queue(EndSynchronizedUpdate).unwrap().flush().unwrap();
    }
}

fn get_term_size() -> (u16, u16) {
    size().unwrap()
}

pub struct VirtualDisplayBuffer {
    // Currently we only support ascii and uncolored... Likely will change.
    pub buf: Vec<char>,
    pub width: u16,
    pub height: u16,
}

#[derive(Resource)]
pub struct TerminalDisplayBuffer(VirtualDisplayBuffer);
impl TerminalDisplayBuffer {
    fn init_from_screen() -> Self {
        let (width, height) = get_term_size();
        log::info!("w,h: {:?},{:?}", width, height);
        Self (VirtualDisplayBuffer{
            buf: vec!['\0'; width as usize * height as usize],
            width: width,
            height: height,
        })
    }
}