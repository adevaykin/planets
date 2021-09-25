mod gameloop;
mod world;

use crate::gameloop::GameLoop;
use crate::world::world::World;

use chrono::Utc;
use log::LevelFilter;
use simplelog::*;


extern crate log_panics;

fn main() {
    log_panics::init(); // Initialize logging of Rust panics to log files in addition to stdout
    init_log();

    log::info!("Starting Planets!");

    let mut gameloop = GameLoop::new();
    gameloop.set_max_fps(2);
    
    let mut fame_num = 0 as u64;
    
    let mut world = World::new();

    loop {
        // Notify frame start
        gameloop.start_frame();

        // All the game logic entry point is here.

        world.update(gameloop.get_prev_frame_time());

        // Update single planet block object here using e.g. gameloop.get_prev_frame_time() function to get passed time
        log::info!("Frame {} started.", fame_num);
        
        
        log::info!("World status: {}", world.get_description_string());
        
        // Increase frame count in the end
        fame_num += 1;
        
    }
}

/// Configure logger to write log to console and a separate log file for every execution
fn init_log() {
    let log_dir = std::path::Path::new("./log");
    std::fs::create_dir_all(log_dir).expect("Could not create log directory.");

    let now = Utc::now();
    let mut filename = now.timestamp().to_string();
    filename.push_str("_planets.log");
    let file_path = log_dir.join(filename);

    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, Config::default(), std::fs::File::create(file_path).expect("Could not create log file.")),
        ]
    ).unwrap();
}
