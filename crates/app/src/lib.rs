mod util;
mod terminal;
pub mod prelude {
    pub use bevy::prelude::*;
}

use prelude::*;
use bevy::{app::ScheduleRunnerSettings, utils::Duration};


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
        .run();
    log::info!("exited app");
}