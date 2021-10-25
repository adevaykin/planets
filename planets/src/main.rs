mod gameloop;
mod system;
mod world;
mod util;
mod engine;
mod vulkan;
mod passes;
mod app;

use app::App;

use winit::event_loop::EventLoop;

extern crate log_panics;

fn main() {
    log_panics::init(); // Initialize logging of Rust panics to log files in addition to stdout
    util::log::init_log();

    log::info!("Starting Planets!");
    let event_loop = EventLoop::new();
    let app = App::new(&event_loop);
    app.run(event_loop);

    log::info!("Application terminated.");
}
