mod gameloop;
mod world;
mod vulkan;
mod engine;
mod util;
mod passes;

use crate::gameloop::GameLoop;
use crate::world::world::World;

use chrono::Utc;
use log::LevelFilter;
use simplelog::*;
use winit::window::WindowBuilder;
use winit::event_loop::{ControlFlow,EventLoop};
use winit::event::{Event, WindowEvent};


extern crate log_panics;

fn main() {
    log_panics::init(); // Initialize logging of Rust panics to log files in addition to stdout
    init_log();

    log::info!("Starting Planets!");

    let mut gameloop = GameLoop::new();
    gameloop.set_max_fps(2);
    
    let mut world = World::new();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let vulkan_resource = vulkan::resource::Resource::new(&window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll; // Continuously poll events even if OS did not provide any

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::info!("Exit requested by window.");
                *control_flow = ControlFlow::Exit
            },
            // Window input events were processed - time to start game loop cycle
            Event::MainEventsCleared => {
                // Events may come too soon due to multiple reasons. Ignore update in such cases.
                if !gameloop.should_start_frame() {
                    return;
                }

                gameloop.start_frame();
                log::info!("Frame {} started.", gameloop.get_frame_num());
                world.update(gameloop.get_prev_frame_time());
                log::info!("World status: {}", world.get_description_string());

                window.request_redraw();
            },
            // Window redraw request came in - time to draw
            Event::RedrawRequested {
                ..
            } => {
                if !gameloop.get_frame_started() {
                    return;
                }
            },
            // Drawing ended - finish frame
            Event::RedrawEventsCleared => {
                gameloop.finish_frame();
                *control_flow = ControlFlow::WaitUntil(gameloop.get_wait_instant());
            }
            _ => (),
        }
    });
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
