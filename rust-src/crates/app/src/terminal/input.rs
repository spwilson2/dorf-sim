use bevy::app::AppExit;

use crate::prelude::*;
use crossterm::event::{poll, read, Event, KeyEvent};
use std::collections::VecDeque;
use std::sync::Mutex;

use bevy::input::keyboard::KeyCode as BevyKeyCode;
use bevy::input::keyboard::{ButtonState, KeyboardInput};
use crossterm::event::KeyCode;
use once_cell::sync::Lazy;

use crate::util::on_exit::RegisterOnExit;

#[derive(Default)]
pub struct TerminalInputPlugin {}
use std::thread::JoinHandle;

impl Plugin for TerminalInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<KeyboardInput>()
            .add_event::<TerminalResize>()
            .add_system(handle_input_buffer)
            .add_system(escape_listener)
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
        events.push(res.clone());
        res.state = ButtonState::Released;
        events.push(res);
    }
    input_writer.send_batch(events);

    if let Some(resize) = input_buf.resize.take() {
        resize_writer.send(resize);
    }
}

fn terminal_keycode_to_bevy(in_code: &crossterm::event::KeyCode) -> Option<BevyKeyCode> {
    Some(match in_code {
        KeyCode::Backspace => BevyKeyCode::Back,
        KeyCode::Enter => BevyKeyCode::Return,
        KeyCode::Left => BevyKeyCode::Left,
        KeyCode::Right => BevyKeyCode::Right,
        KeyCode::Up => BevyKeyCode::Up,
        KeyCode::Down => BevyKeyCode::Down,
        KeyCode::Home => BevyKeyCode::Home,
        KeyCode::End => BevyKeyCode::End,
        KeyCode::PageUp => BevyKeyCode::PageUp,
        KeyCode::PageDown => BevyKeyCode::PageDown,
        KeyCode::Tab => BevyKeyCode::Tab,
        KeyCode::BackTab => panic!(),
        KeyCode::Delete => BevyKeyCode::Delete,
        KeyCode::Insert => BevyKeyCode::Insert,
        KeyCode::F(u8) => todo!(),
        KeyCode::Char(c) => charcode_to_bevy_key_code(*c),
        KeyCode::Null => todo!(),
        KeyCode::Esc => BevyKeyCode::Escape,
        KeyCode::CapsLock => todo!(),
        KeyCode::ScrollLock => todo!(),
        KeyCode::NumLock => BevyKeyCode::Numlock,
        KeyCode::PrintScreen => todo!(),
        KeyCode::Pause => BevyKeyCode::Pause,
        KeyCode::Menu => todo!(),
        KeyCode::KeypadBegin => todo!(),
        KeyCode::Media(media_key_codee) => todo!(),
        KeyCode::Modifier(modifier_key_code) => todo!(),
    })
}

fn charcode_to_bevy_key_code(c: char) -> BevyKeyCode {
    match c {
        '1' => BevyKeyCode::Key1,
        '2' => BevyKeyCode::Key2,
        '3' => BevyKeyCode::Key3,
        '4' => BevyKeyCode::Key4,
        '5' => BevyKeyCode::Key5,
        '6' => BevyKeyCode::Key6,
        '7' => BevyKeyCode::Key7,
        '8' => BevyKeyCode::Key8,
        '9' => BevyKeyCode::Key9,
        '0' => BevyKeyCode::Key0,
        'A' => BevyKeyCode::A,
        'B' => BevyKeyCode::B,
        'C' => BevyKeyCode::C,
        'D' => BevyKeyCode::D,
        'E' => BevyKeyCode::E,
        'F' => BevyKeyCode::F,
        'G' => BevyKeyCode::G,
        'H' => BevyKeyCode::H,
        'I' => BevyKeyCode::I,
        'J' => BevyKeyCode::J,
        'K' => BevyKeyCode::K,
        'L' => BevyKeyCode::L,
        'M' => BevyKeyCode::M,
        'N' => BevyKeyCode::N,
        'O' => BevyKeyCode::O,
        'P' => BevyKeyCode::P,
        'Q' => BevyKeyCode::Q,
        'R' => BevyKeyCode::R,
        'S' => BevyKeyCode::S,
        'T' => BevyKeyCode::T,
        'U' => BevyKeyCode::U,
        'V' => BevyKeyCode::V,
        'W' => BevyKeyCode::W,
        'X' => BevyKeyCode::X,
        'Y' => BevyKeyCode::Y,
        'Z' => BevyKeyCode::Z,
        //'[' => BevKeyCode::
        //'\\' => BevKeyCode::Backslash,
        //']' => BevKeyCode::
        //'^' => BevKeyCode::
        //'_' => BevKeyCode::
        //'`' => BevKeyCode::
        'a' => BevyKeyCode::A,
        'b' => BevyKeyCode::B,
        'c' => BevyKeyCode::C,
        'd' => BevyKeyCode::D,
        'e' => BevyKeyCode::E,
        'f' => BevyKeyCode::F,
        'g' => BevyKeyCode::G,
        'h' => BevyKeyCode::H,
        'i' => BevyKeyCode::I,
        'j' => BevyKeyCode::J,
        'k' => BevyKeyCode::K,
        'l' => BevyKeyCode::L,
        'm' => BevyKeyCode::M,
        'n' => BevyKeyCode::N,
        'o' => BevyKeyCode::O,
        'p' => BevyKeyCode::P,
        'q' => BevyKeyCode::Q,
        'r' => BevyKeyCode::R,
        's' => BevyKeyCode::S,
        't' => BevyKeyCode::T,
        'u' => BevyKeyCode::U,
        'v' => BevyKeyCode::V,
        'w' => BevyKeyCode::W,
        'x' => BevyKeyCode::X,
        'y' => BevyKeyCode::Y,
        'z' => BevyKeyCode::Z,
        //'{' => BevKeyCode::
        //'|' => BevKeyCode::
        //'}' => BevKeyCode::
        //'~' => BevKeyCode::
        // '\'' => BevyKeyCode::
        _ => todo!(),
    }
}

fn escape_listener(mut input: EventReader<KeyboardInput>, mut writer: EventWriter<AppExit>) {
    for e in input.iter() {
        if let Some(k) = e.key_code {
            if [BevyKeyCode::Escape, BevyKeyCode::Q].contains(&k) {
                writer.send(AppExit);
            }
        }
    }
}

fn init(onexit_register: EventWriter<RegisterOnExit>) {
    // Spawn IO thread which reads and buffers input, we'll check every frame for input.
    INPUT_THREAD_BUF.lock().unwrap().handle = Some(std::thread::spawn(input_thread_loop));
}
