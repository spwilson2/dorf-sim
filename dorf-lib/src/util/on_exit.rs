use crate::prelude::*;
use bevy::app::AppExit;
use once_cell::sync::Lazy;
use shutdown_hooks::add_shutdown_hook;
use std::sync::{atomic::AtomicBool, Arc, Mutex};

pub type Callback = fn() -> ();
pub struct RegisterOnExit(pub Callback);

static CALLBACKS: Lazy<Mutex<Vec<Callback>>> = Lazy::new(|| Mutex::new(vec![]));
static SIGTERM_SIGNAL: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

// Note; Even though we use the static variable, we define this Resource to
// prevent contention between users.
#[derive(Resource)]
struct OnExitCallbacks {}

pub struct OnExitPlugin {}

impl Plugin for OnExitPlugin {
    fn build(&self, app: &mut App) {
        add_shutdown_hook(on_exit);

        app.insert_resource(OnExitCallbacks {})
            .add_event::<RegisterOnExit>()
            .add_system(check_sigterm_signal)
            .add_system(handle_register_onexit)
            .add_system(handle_app_exit)
            .add_system(handle_onexit);

        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&SIGTERM_SIGNAL))
            .unwrap();
    }
}

fn check_sigterm_signal(mut exit: EventWriter<AppExit>) {
    if SIGTERM_SIGNAL.load(std::sync::atomic::Ordering::Relaxed) {
        log::info!("CTLR-C");
        exit.send(AppExit)
    }
}

extern "C" fn on_exit() {
    for cb in (*CALLBACKS.lock().unwrap()).drain(0..) {
        cb()
    }
}

// We attempt to cleanly handle the app exiting and only rely on the libc::atexit behavior if we strictly need to.
fn handle_app_exit(mut _callbacks: ResMut<OnExitCallbacks>, ev_recv: EventReader<AppExit>) {
    if !ev_recv.is_empty() {
        on_exit();
    }
}

fn handle_register_onexit(
    mut _callbacks: ResMut<OnExitCallbacks>,
    mut ev_recv: EventReader<RegisterOnExit>,
) {
    if !ev_recv.is_empty() {
        let mut cbs = (*CALLBACKS).lock().unwrap();
        for ev in ev_recv.iter() {
            cbs.push(ev.0);
        }
    }
}

fn handle_onexit() {}
