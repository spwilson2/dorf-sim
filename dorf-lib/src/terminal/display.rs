use std::io::{stdout, StdoutLock, Write};

use bytemuck::checked::try_cast_slice;
use crossterm::cursor::MoveTo;
use crossterm::style::{Color, Stylize};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, BeginSynchronizedUpdate, Clear, ClearType,
    EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen, SetSize,
};
use crossterm::{execute, QueueableCommand};
use crossterm::{queue, style};

use crate::prelude::*;

use super::input::TerminalResize;

#[derive(Default)]
pub struct TerminalDisplayPlugin {}

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct TerminalDisplayBuffer(DisplayBuffer);

#[derive(Debug, Clone)]
pub struct DisplayBuffer {
    // NOTE/TODO: Will want to add support for converting from RGB true color to 256
    // https://www.ditig.com/256-colors-cheat-sheet
    pub texture_vec: Vec<CharTexture>,
    pub width: u16,
    pub height: u16,
}

impl TerminalDisplayBuffer {
    pub fn get_mut_dbg_checked(&mut self, x: usize, y: usize) -> &mut CharTexture {
        let width = self.width as usize;
        #[cfg(debug_assertions)]
        {
            return self.texture_vec.get_mut(x + y * width).unwrap();
        }
        #[cfg(not(debug_assertions))]
        return unsafe { self.texture_vec.get_mut(x + y * width).unwrap_unchecked() };
    }
}

/************************************************************************************************ */
/*                      Private implementation below...                                         * */
/************************************************************************************************ */

impl Plugin for TerminalDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init)
            .insert_resource(TerminalDisplayBuffer(DisplayBuffer::init_from_screen()))
            .insert_resource(PhysicalDisplayBuffer::new())
            .add_system(handle_terminal_resize)
            .add_system(sys_display_paint);
    }
}

impl DisplayBuffer {
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.texture_vec.clear();
        self.texture_vec
            .resize(width as usize * height as usize, default());
    }
    pub fn reinit(&mut self) {
        let width = self.width;
        let height = self.height;
        self.resize(width, height);
    }
}

#[derive(Resource, Debug)]
struct PhysicalDisplayBuffer {
    buf: DisplayBuffer,
    need_flush: bool,
}

impl std::ops::DerefMut for PhysicalDisplayBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}
impl std::ops::Deref for PhysicalDisplayBuffer {
    type Target = DisplayBuffer;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl DisplayBuffer {
    fn init_from_screen() -> Self {
        let (width, height) = get_term_size();
        let buf = DisplayBuffer {
            texture_vec: vec![default(); width as usize * height as usize],
            width,
            height,
        };
        buf
    }
}

impl PhysicalDisplayBuffer {
    fn new() -> Self {
        Self {
            buf: DisplayBuffer::init_from_screen(),
            need_flush: true,
        }
    }
}

/// Initalize the display by setting it to raw mode and moving to the alternate
/// screen. Also register a cleanup handler to restore settings on
/// [`AppExit`]/panic.
fn init(mut onexit_register: EventWriter<RegisterOnExit>) {
    enable_raw_mode().unwrap();
    execute!(stdout(), EnterAlternateScreen, crossterm::cursor::Hide,).unwrap();

    onexit_register.send(RegisterOnExit(cleanup));
}

/// Undo [`init`].
fn cleanup() {
    log::info!("Performing terminal cleanup");
    disable_raw_mode().unwrap();
    execute!(stdout(), LeaveAlternateScreen, crossterm::cursor::Show).unwrap();
}

/// Convert our own RGB implementation into crossterm's
#[inline]
fn rgb_convert(ours: RGB) -> Color {
    Color::Rgb {
        r: ours.r,
        g: ours.g,
        b: ours.b,
    }
}

#[inline]
fn write_texture(stdout: &mut StdoutLock, texture: &CharTexture) {
    let mut binding = [0; 4];
    let c = texture.c.encode_utf8(&mut binding);
    if let Some(rgb) = texture.rgb {
        stdout.queue(style::PrintStyledContent(c.with(rgb_convert(rgb))));
    } else {
        stdout.write(c.as_bytes());
    }
}

/// Handler for [`TerminalResize`] events, updates buffer sizes to match the new
/// screen size. Resize events will introduce a full repaint.
fn handle_terminal_resize(
    mut resize_reader: EventReader<TerminalResize>,
    mut virt_term_buffer: ResMut<TerminalDisplayBuffer>,
    mut phys_term_buffer: ResMut<PhysicalDisplayBuffer>,
) {
    if let Some(resize) = resize_reader.iter().last() {
        virt_term_buffer.resize(resize.width, resize.height);
        phys_term_buffer.buf.resize(resize.width, resize.height);
        // Resize events will fk shit up, we'll need to repaint.
        phys_term_buffer.need_flush = true;
    }
}

/// Utility function to handle painting simply clearign the entire physical
/// buffer and repainting the full screen. This is required on resize events.
fn paint_all(
    virt_term_buffer: Res<TerminalDisplayBuffer>,
    mut phys_term_buffer: ResMut<PhysicalDisplayBuffer>,
) {
    // Prevent anything else from interrupting our paint, hold stdout lock until
    // we're done.
    let mut stdout = stdout().lock();
    queue!(
        stdout,
        BeginSynchronizedUpdate,
        MoveTo(0, 0),
        Clear(ClearType::All)
    )
    .unwrap();

    // If we're flushing, clear the backing buffer, this will cause us to reinitialize it and write new data.
    //phys_term_buffer.buf.reinit();
    phys_term_buffer.buf = virt_term_buffer.0.clone();

    for texture in virt_term_buffer.texture_vec.iter() {
        write_texture(&mut stdout, texture);
    }

    stdout
        .queue(EndSynchronizedUpdate)
        .unwrap()
        .flush()
        .unwrap();
    phys_term_buffer.need_flush = false;
}

/// System to handle moving the virtual display buffer cache into a physical
/// buffer which will then be painted to the terminal.
///
/// Note that even "physical buffer" isn't really a direct buffer into the
/// terminal. That is, the terminal doesn't actually have access to the physical
/// buffer. We use the physical buffer to maintain what we believe the state of
/// the actual terminal looks like.
fn sys_display_paint(
    virt_term_buffer: Res<TerminalDisplayBuffer>,
    mut phys_term_buffer: ResMut<PhysicalDisplayBuffer>,
) {
    // Detect if there's an update if not skip the paint.
    if !virt_term_buffer.is_changed() {
        return;
    }
    // For now, let's just check if  the dimmensions look like they're gonna be
    // fkd and log a warning, we can updated/fix in the next pass.
    #[cfg(debug_assertions)]
    {
        let (width, height) = get_term_size();
        if (width, height) != (virt_term_buffer.width, virt_term_buffer.height) {
            log::warn!(
                "Write buffer size: {:?} doesn't match current terminal size: {:?}",
                (virt_term_buffer.width, virt_term_buffer.height),
                (width, height)
            );
        }
    }
    debug_assert_eq!(
        phys_term_buffer.texture_vec.len(),
        virt_term_buffer.texture_vec.len()
    );
    debug_assert_eq!(
        virt_term_buffer.texture_vec.len(),
        virt_term_buffer.width as usize * virt_term_buffer.height as usize
    );

    if phys_term_buffer.need_flush {
        paint_all(virt_term_buffer, phys_term_buffer);
        return;
    }

    if virt_term_buffer.texture_vec == phys_term_buffer.texture_vec {
        return;
    }

    let mut stdout = stdout().lock();
    queue!(
        stdout,
        BeginSynchronizedUpdate,
        MoveTo(0, 0),
        // I don't know what this SetSize would actually do.. will disable for
        // now. Just came fromt the example input for crossterm...
        // SetSize(width, height),
    )
    .unwrap();

    // Now just iterate, write in only changes...
    for (idx, (v_c, p_c_mut)) in virt_term_buffer
        .texture_vec
        .iter()
        .zip(phys_term_buffer.texture_vec.iter_mut())
        .enumerate()
    {
        if *v_c != *p_c_mut {
            let col = idx % virt_term_buffer.width as usize;
            let row = idx / virt_term_buffer.width as usize;
            // Move cursor and write
            write_texture(stdout.queue(MoveTo(col as u16, row as u16)).unwrap(), v_c);
            // Update phys buffer
            *p_c_mut = v_c.clone();
        }
    }

    #[cfg(all(debug_assertions, not(feature = "no_expensive_assertions")))]
    assert_eq!(phys_term_buffer.c_vec, virt_term_buffer.c_vec);

    stdout
        .queue(EndSynchronizedUpdate)
        .unwrap()
        .flush()
        .unwrap();
}

fn get_term_size() -> (u16, u16) {
    size().unwrap()
}
