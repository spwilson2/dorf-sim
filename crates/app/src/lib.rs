//! This example will display a simple menu using Bevy UI where you can start a new game,
//! change some settings or quit. There is no actual game, it will just display the current
//! settings for 5 seconds before going back to the menu.

mod ecs;
pub mod prelude {
    pub use crate::ecs::*;
    pub use bevy::prelude::*;
}

pub mod onexit {
    use crate::prelude::*;
    use std::sync::Mutex;
    use bevy::app::AppExit;
    use shutdown_hooks::add_shutdown_hook;
    use once_cell::sync::Lazy;

    pub type Callback = fn() -> ();
    pub struct RegisterOnExit(pub Callback);

    static CALLBACKS: Lazy<Mutex<Vec<Callback>>> = Lazy::new(|| Mutex::new(vec![]));

    // Note; Even though we use the static variable, we define this Resource to
    // prevent contention between users.
    #[derive(Resource)]
    struct OnExitCallbacks { }

    pub struct OnExitPlugin{}

    impl Plugin for OnExitPlugin {
        fn build(&self, app: &mut App) {
            add_shutdown_hook(on_exit);
            app
            .insert_resource(OnExitCallbacks {})
            .add_event::<RegisterOnExit>()
            .add_system(handle_register_onexit)
            .add_system(handle_app_exit)
            .add_system(handle_onexit);

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

    fn handle_register_onexit(mut _callbacks: ResMut<OnExitCallbacks>, mut ev_recv: EventReader<RegisterOnExit>) {
        if !ev_recv.is_empty() {
            let mut cbs = (*CALLBACKS).lock().unwrap();
            for ev in ev_recv.iter() {
                cbs.push(ev.0);
            }
        }
    }

    fn handle_onexit() {}
}


use bevy::{app::ScheduleRunnerSettings, utils::Duration};
use prelude::*;

mod terminal;

fn configure_logging() {
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Root};

    let logfile = FileAppender::builder()
    .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S %Z)(utc)} | {l:<6.6}| {f}:{L} | {m}{n}")))
    .build("log/output.log").unwrap();

    let config = Config::builder()
    .appender(Appender::builder().build("logfile", Box::new(logfile)))
    .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Info)).unwrap();

    log4rs::init_config(config).unwrap();
    log::info!("Log initialized.")
}

pub fn app_main() {
    configure_logging();

    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(MinimalPlugins)
        .add_plugin(terminal::TerminalPlugin::default())
        .add_startup_system(hello_world_system)
        .run();
    log::info!("exited app");
}

fn hello_world_system() {println!("hello")}