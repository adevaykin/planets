use chrono::Utc;
use simplelog::{CombinedLogger, TermLogger, WriteLogger, Config, ColorChoice, TerminalMode};
use log::LevelFilter;

/// Configure logger to write log to console and a separate log file for every execution
pub fn init_log() {
    let log_dir = std::path::Path::new("./log");
    std::fs::create_dir_all(log_dir).expect("Could not create log directory.");

    let now = Utc::now();
    let mut filename = now.timestamp().to_string();
    filename.push_str("_planets.log");
    let file_path = log_dir.join(filename);

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            std::fs::File::create(file_path).expect("Could not create log file."),
        ),
    ])
        .unwrap();
}