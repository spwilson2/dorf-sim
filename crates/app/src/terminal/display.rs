use std::io::{stdout, StdoutLock, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, BeginSynchronizedUpdate, Clear, ClearType,
    EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen, SetSize,
};
use crossterm::{execute, QueueableCommand};

use crate::prelude::*;
use crate::util::on_exit::{OnExitPlugin, RegisterOnExit};

use super::input::TerminalResize;

#[derive(Default)]
pub struct TerminalDisplayPlugin {}

impl Plugin for TerminalDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(OnExitPlugin {})
            .add_startup_system(init)
            .insert_resource(TerminalDisplayBuffer::init_from_screen())
            .add_system(handle_terminal_resize)
            .add_system(paint);
    }
}

fn init(mut onexit_register: EventWriter<RegisterOnExit>) {
    enable_raw_mode().unwrap();
    execute!(stdout(), EnterAlternateScreen, crossterm::cursor::Hide,).unwrap();

    onexit_register.send(RegisterOnExit(cleanup));
}

fn cleanup() {
    log::info!("Performing terminal cleanup");
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show).unwrap();
}

fn handle_terminal_resize(
    mut term_buffer: ResMut<TerminalDisplayBuffer>,
    mut resize_reader: EventReader<TerminalResize>,
) {
    if let Some(resize) = resize_reader.iter().last() {
        term_buffer
            .virtual_frame_mut()
            .resize(resize.width, resize.height);
        term_buffer
            .physical_frame_mut()
            .resize(resize.width, resize.height);
        // Resize events will fk shit up, we'll need to repaint.
        term_buffer.enable_flush();
    }
}

fn paint_all(term_buffer: &mut ResMut<TerminalDisplayBuffer>) {
    let mut stdout = stdout().lock();
    queue!(
        stdout,
        BeginSynchronizedUpdate,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )
    .unwrap();
    // If we're flushing, clear the backing buffer, this will cause us to reinitialize it and write new data.
    term_buffer.physical_frame_mut().buf.clear();
    let (virt, phys) = term_buffer.virt_phys_buffers_mut();

    // Full pass repaint, collect values into the physical buffer as we repaint.
    stdout
        .write(
            virt.buf
                .iter()
                .map(|c| {
                    phys.buf.push(*c);
                    *c as u8
                })
                .collect::<Vec<u8>>()
                .as_slice(),
        )
        .unwrap();
    stdout
        .queue(EndSynchronizedUpdate)
        .unwrap()
        .flush()
        .unwrap();
}

fn paint(mut term_buffer: ResMut<TerminalDisplayBuffer>) {
    // Detect if there's an update.
    // If so, perform the render. (TODO: Maybe only render part if necessary?)
    if term_buffer.is_changed() {
        log::info!("Change detected");
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
                log::warn!(
                    "Write buffer size: {:?} doesn't match current terminal size: {:?}",
                    (term_buffer.0.width, term_buffer.0.height),
                    (width, height)
                );
            }
        }
        debug_assert_eq!(
            term_buffer.physical_frame_ref().buf.len(),
            term_buffer.virtual_frame_ref().buf.len()
        );
        debug_assert_eq!(
            term_buffer.0.buf.len(),
            term_buffer.0.width as usize * term_buffer.0.height as usize
        );

        if term_buffer.get_flush() {
            log::info!("Performing full flush paint.");
            paint_all(&mut term_buffer);
            term_buffer.set_flush(false);
            return;
        }

        let (virt, phys) = term_buffer.virt_phys_buffers_ref();
        if virt.buf == phys.buf {
            return;
        }

        let (virt, phys) = term_buffer.virt_phys_buffers_mut();
        let width = virt.width;
        let height = virt.height;

        let mut stdout = stdout().lock();
        log::info!("Painting!");
        queue!(
            stdout,
            BeginSynchronizedUpdate,
            MoveTo(0, 0),
            // I don't know what this would actually do.. won't bother enabling for now.
            //SetSize(width, height),
        )
        .unwrap();
        // Now just iterate, write in only changes...
        for (idx, (v_c, p_c_mut)) in virt.buf.iter().zip(phys.buf.iter_mut()).enumerate() {
            if *v_c != *p_c_mut {
                let col = idx % width as usize;
                let row = idx / width as usize;
                // Move cursor and write
                stdout
                    .queue(MoveTo(col as u16, row as u16))
                    .unwrap()
                    .write(&[*v_c as u8]);
                // Update phys buffer
                *p_c_mut = *v_c;
            }
        }
        stdout
            .queue(EndSynchronizedUpdate)
            .unwrap()
            .flush()
            .unwrap();
    }
}

fn get_term_size() -> (u16, u16) {
    size().unwrap()
}

#[derive(Clone)]
pub struct VirtualDisplayBuffer {
    // Currently we only support ascii and uncolored... Likely will change.
    pub buf: Vec<char>,
    pub width: u16,
    pub height: u16,
}

impl VirtualDisplayBuffer {
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.buf.clear();
        self.buf.resize((width * height) as usize, ' ');
    }
}

#[derive(Resource)]
pub struct TerminalDisplayBuffer(
    pub VirtualDisplayBuffer,
    // Second buffer is used for tracking the actual state internally.
    // We will only redraw what we need to.
    pub(self) VirtualDisplayBuffer,
    bool,
);
impl TerminalDisplayBuffer {
    fn init_from_screen() -> Self {
        let (width, height) = get_term_size();
        log::info!("w,h: {:?},{:?}", width, height);
        let buf = VirtualDisplayBuffer {
            buf: vec!['\0'; width as usize * height as usize],
            width,
            height,
        };
        Self(buf.clone(), buf, true)
    }

    pub fn enable_flush(&mut self) {
        self.2 = true;
    }

    pub fn get_flush(&self) -> bool {
        self.2
    }

    fn set_flush(&mut self, val: bool) {
        self.2 = val
    }

    fn physical_frame_mut(&mut self) -> &mut VirtualDisplayBuffer {
        &mut self.1
    }
    fn physical_frame_ref(&self) -> &VirtualDisplayBuffer {
        &self.1
    }

    fn virt_phys_buffers_mut(&mut self) -> (&mut VirtualDisplayBuffer, &mut VirtualDisplayBuffer) {
        (&mut self.0, &mut self.1)
    }

    fn virt_phys_buffers_ref(&self) -> (&VirtualDisplayBuffer, &VirtualDisplayBuffer) {
        (&self.0, &self.1)
    }

    pub fn virtual_frame_mut(&mut self) -> &mut VirtualDisplayBuffer {
        &mut self.0
    }

    pub fn virtual_frame_ref(&self) -> &VirtualDisplayBuffer {
        &self.0
    }
}
