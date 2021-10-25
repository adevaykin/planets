mod gameloop;
mod system;
mod world;
mod util;
mod engine;
mod vulkan;
mod passes;
mod app;

use crate::gameloop::GameLoop;
use crate::system::serialize::{Saver, Loader};
use crate::world::world::World;
use app::App;

use chrono::Utc;
use log::LevelFilter;
use simplelog::*;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder, Window};

use crate::util::helpers::SimpleViewportSize;
use ash::vk;
use crate::passes::background::BackgroundPass;
use std::rc::Rc;
use std::cell::RefCell;
use crate::engine::camera::{Camera, CameraMutRef};
use crate::vulkan::device::MAX_FRAMES_IN_FLIGHT;
use crate::engine::timer::{TimerMutRef, Timer};
use crate::engine::renderer::Renderer;
use crate::engine::viewport::Viewport;
use winit::dpi::PhysicalSize;
use crate::util::constants::{WINDOW_WIDTH, WINDOW_HEIGHT};

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
