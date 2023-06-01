use crate::prelude::*;
use crate::util::on_exit::RegisterOnExit;
use bevy::input::ButtonState;
use crossterm::event::KeyCode as CrosstermKeyCode;
use crossterm::event::{poll, read, Event, KeyEvent};
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;

pub use bevy::input::keyboard::KeyCode;
pub use bevy::input::keyboard::KeyboardInput;

#[derive(Default)]
pub struct TerminalInputPlugin {}
use std::thread::JoinHandle;

impl Plugin for TerminalInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<KeyboardInput>()
            .add_event::<TerminalResize>()
            .add_system(handle_input_buffer)
            .add_startup_system(init);
    }
}

#[derive(Default)]
struct TerminalState {
    handle: Option<JoinHandle<()>>,
    key_buffer: VecDeque<KeyEvent>,
    resize: Option<TerminalResize>,
}

fn input_thread_loop() {
    loop {
        if poll(std::time::Duration::from_millis(500)).unwrap() {
            // It's guaranteed that the `read()` won't block when the `poll()` function returns `true`
            match read().unwrap() {
                Event::Key(event) => INPUT_THREAD_BUF
                    .lock()
                    .unwrap()
                    .key_buffer
                    .push_front(event),
                Event::Resize(width, height) => {
                    INPUT_THREAD_BUF.lock().unwrap().resize = Some(TerminalResize { width, height })
                }
                _ => (),
            }
        } else {
            // Timeout expired and no `Event` is available
        }
    }
}

static INPUT_THREAD_BUF: Lazy<Mutex<TerminalState>> = Lazy::new(|| {
    Mutex::new(TerminalState {
        handle: None,
        key_buffer: VecDeque::default(),
        resize: None,
    })
});

#[derive(Debug, Default, Clone)]
pub struct TerminalResize {
    pub width: u16,
    pub height: u16,
}

fn handle_input_buffer(
    mut input_writer: EventWriter<KeyboardInput>,
    mut resize_writer: EventWriter<TerminalResize>,
) {
    let mut input_buf = INPUT_THREAD_BUF.lock().unwrap();

    let mut events = Vec::new();
    for event in input_buf.key_buffer.drain(0..) {
        // TODO Process.
        //event_writer.send(KeyInputEvent { key: event.code });
        let mut res = KeyboardInput {
            scan_code: 0, /* TODO, not included by vanilla termion. */
            key_code: terminal_keycode_to_bevy(&event.code),
            state: ButtonState::Pressed,
        };
        events.push(res);
        res.state = ButtonState::Released;
        events.push(res);
    }
    input_writer.send_batch(events);

    if let Some(resize) = input_buf.resize.take() {
        resize_writer.send(resize);
    }
}

fn terminal_keycode_to_bevy(in_code: &CrosstermKeyCode) -> Option<KeyCode> {
    Some(match in_code {
        CrosstermKeyCode::Backspace => KeyCode::Back,
        CrosstermKeyCode::Enter => KeyCode::Return,
        CrosstermKeyCode::Left => KeyCode::Left,
        CrosstermKeyCode::Right => KeyCode::Right,
        CrosstermKeyCode::Up => KeyCode::Up,
        CrosstermKeyCode::Down => KeyCode::Down,
        CrosstermKeyCode::Home => KeyCode::Home,
        CrosstermKeyCode::End => KeyCode::End,
        CrosstermKeyCode::PageUp => KeyCode::PageUp,
        CrosstermKeyCode::PageDown => KeyCode::PageDown,
        CrosstermKeyCode::Tab => KeyCode::Tab,
        CrosstermKeyCode::BackTab => panic!(),
        CrosstermKeyCode::Delete => KeyCode::Delete,
        CrosstermKeyCode::Insert => KeyCode::Insert,
        CrosstermKeyCode::F(u8) => todo!(),
        CrosstermKeyCode::Char(c) => charcode_to_bevy_key_code(*c),
        CrosstermKeyCode::Null => todo!(),
        CrosstermKeyCode::Esc => KeyCode::Escape,
        CrosstermKeyCode::CapsLock => todo!(),
        CrosstermKeyCode::ScrollLock => todo!(),
        CrosstermKeyCode::NumLock => KeyCode::Numlock,
        CrosstermKeyCode::PrintScreen => todo!(),
        CrosstermKeyCode::Pause => KeyCode::Pause,
        CrosstermKeyCode::Menu => todo!(),
        CrosstermKeyCode::KeypadBegin => todo!(),
        CrosstermKeyCode::Media(media_key_codee) => todo!(),
        CrosstermKeyCode::Modifier(modifier_key_code) => todo!(),
    })
}

fn charcode_to_bevy_key_code(c: char) -> KeyCode {
    match c {
        '1' => KeyCode::Key1,
        '2' => KeyCode::Key2,
        '3' => KeyCode::Key3,
        '4' => KeyCode::Key4,
        '5' => KeyCode::Key5,
        '6' => KeyCode::Key6,
        '7' => KeyCode::Key7,
        '8' => KeyCode::Key8,
        '9' => KeyCode::Key9,
        '0' => KeyCode::Key0,
        'A' => KeyCode::A,
        'B' => KeyCode::B,
        'C' => KeyCode::C,
        'D' => KeyCode::D,
        'E' => KeyCode::E,
        'F' => KeyCode::F,
        'G' => KeyCode::G,
        'H' => KeyCode::H,
        'I' => KeyCode::I,
        'J' => KeyCode::J,
        'K' => KeyCode::K,
        'L' => KeyCode::L,
        'M' => KeyCode::M,
        'N' => KeyCode::N,
        'O' => KeyCode::O,
        'P' => KeyCode::P,
        'Q' => KeyCode::Q,
        'R' => KeyCode::R,
        'S' => KeyCode::S,
        'T' => KeyCode::T,
        'U' => KeyCode::U,
        'V' => KeyCode::V,
        'W' => KeyCode::W,
        'X' => KeyCode::X,
        'Y' => KeyCode::Y,
        'Z' => KeyCode::Z,
        //'[' => BevKeyCode::
        //'\\' => BevKeyCode::Backslash,
        //']' => BevKeyCode::
        //'^' => BevKeyCode::
        //'_' => BevKeyCode::
        //'`' => BevKeyCode::
        'a' => KeyCode::A,
        'b' => KeyCode::B,
        'c' => KeyCode::C,
        'd' => KeyCode::D,
        'e' => KeyCode::E,
        'f' => KeyCode::F,
        'g' => KeyCode::G,
        'h' => KeyCode::H,
        'i' => KeyCode::I,
        'j' => KeyCode::J,
        'k' => KeyCode::K,
        'l' => KeyCode::L,
        'm' => KeyCode::M,
        'n' => KeyCode::N,
        'o' => KeyCode::O,
        'p' => KeyCode::P,
        'q' => KeyCode::Q,
        'r' => KeyCode::R,
        's' => KeyCode::S,
        't' => KeyCode::T,
        'u' => KeyCode::U,
        'v' => KeyCode::V,
        'w' => KeyCode::W,
        'x' => KeyCode::X,
        'y' => KeyCode::Y,
        'z' => KeyCode::Z,
        //'{' => BevKeyCode::
        //'|' => BevKeyCode::
        //'}' => BevKeyCode::
        //'~' => BevKeyCode::
        // '\'' => KeyCode::
        _ => todo!(),
    }
}

fn init(onexit_register: EventWriter<RegisterOnExit>) {
    // Spawn IO thread which reads and buffers input, we'll check every frame for input.
    INPUT_THREAD_BUF.lock().unwrap().handle = Some(std::thread::spawn(input_thread_loop));
}
