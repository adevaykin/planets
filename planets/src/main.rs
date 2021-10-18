mod gameloop;
mod system;
mod world;
mod util;
mod engine;
mod vulkan;
mod passes;

use crate::gameloop::GameLoop;
use crate::system::serialize::{Saver, Loader};
use crate::world::world::World;

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

extern crate log_panics;

struct App {
    gameloop: GameLoop,
    world: World,
    window: Window,
    vulkan: vulkan::entry::Entry,
    is_paused: bool,
    timer: TimerMutRef,
    camera: CameraMutRef,
    renderer: Renderer,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> Self {
        let mut gameloop = GameLoop::new();
        gameloop.set_max_fps(2);

        let world = World::new();

        let window = WindowBuilder::new()
            .with_title("2.5B Initiative: Planets")
            .build(event_loop).unwrap();
        let vulkan = vulkan::entry::Entry::new(&window);
        // TODO: remove camera instantiation from here
        let camera = Rc::new(RefCell::new(Camera::new(&mut vulkan.get_resource_manager().borrow_mut())));
        // TODO: move timer to some other place from here
        let timer = Rc::new(RefCell::new(Timer::new(&mut vulkan.get_resource_manager().borrow_mut())));
        let viewport = Rc::new(RefCell::new(Viewport::new(window.inner_size().width, window.inner_size().height)));
        let background_pass = Box::new(BackgroundPass::new(&vulkan.get_device(), vulkan.get_resource_manager(), &timer, vulkan.get_shader_manager(), &viewport, &camera, "Background"));

        let mut renderer = Renderer::new(vulkan.get_device(), &vulkan.get_resource_manager(), &viewport.borrow());
        renderer.add_pass(background_pass);

        App {
            gameloop,
            world,
            window,
            vulkan,
            is_paused: false,
            timer,
            camera,
            renderer
        }
    }

    fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll; // Continuously poll events even if OS did not provide any

            match event {
                Event::WindowEvent {
                    event, ..
                } => match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Exit requested by window close request.");
                        self.is_paused = true;
                        self.vulkan.get_device().borrow().wait_idle();
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::Destroyed => {
                        log::info!("Exit on window destruction.");
                        self.is_paused = true;
                        self.vulkan.get_device().borrow().wait_idle();
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::S),
                            state: ElementState::Released,
                            ..
                        } => {

                            let saver = Saver::new();
                            saver.save(&self.world);
                        },
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::L),
                            state: ElementState::Released,
                            ..
                        } => {

                            let loader = Loader::new();
                            self.world = loader.load();
                        },
                        _ => {}
                    },
                    _ => {}
                },
                // Window input events were processed - time to start game loop cycle
                Event::MainEventsCleared => {
                    // Events may come too soon due to multiple reasons. Ignore update in such cases.
                    if !self.gameloop.should_start_frame() {
                        return;
                    }

                    self.gameloop.start_frame();
                    self.update_game();
                    self.window.request_redraw();
                },
                // Window redraw request came in - time to draw
                Event::RedrawRequested {
                    ..
                } => {
                    if self.is_paused || !self.gameloop.get_frame_started() {
                        return;
                    }

                    match self.vulkan.get_mut_swapchain().acquire_next_image() {
                        Ok(image_idx) => {
                            self.vulkan.start_frame(image_idx);
                            self.draw_frame(image_idx);
                            self.renderer.render(image_idx);
                        },
                        Err(_) => {
                            let window_size = self.window.inner_size();
                            self.vulkan.recreate_swapchain(window_size.width, window_size.height);
                            // TODO: recreate render passes here?
                        }
                    }
                },
                // Drawing ended - finish frame
                Event::RedrawEventsCleared => {
                    self.gameloop.finish_frame();
                    *control_flow = ControlFlow::WaitUntil(self.gameloop.get_wait_instant());
                },
                | Event::Suspended => {
                    self.is_paused = true;
                },
                | Event::Resumed => {
                    self.is_paused = false;
                }
                _ => (),
            }
        });
    }

    fn update_game(&mut self) {
        log::info!("Frame {} started.", self.gameloop.get_frame_num());
        self.world.update(self.gameloop.get_prev_frame_time());
        log::info!("World status: {}", self.world.get_description_string());
    }

    fn draw_frame(&mut self, frame_idx: usize) {
        self.timer.borrow_mut().update(&self.gameloop, &self.vulkan.get_device().borrow());
        let viewport_size = SimpleViewportSize {
            offset_x: 0.0,
            offset_y: 0.0,
            width: self.window.inner_size().width as f32,
            height: self.window.inner_size().height as f32
        };
        self.camera.borrow_mut().update(&self.vulkan.get_device().borrow(), &viewport_size);

        self.renderer.render(frame_idx);

        //self.draw_list.borrow_mut().cull(frame_num, &self.scene.borrow_mut());

        self.vulkan.get_swapchain().submit(&self.vulkan.get_device().borrow().command_buffers[frame_idx]);
        //self.draw_list.borrow_mut().end_frame(frame_num);
        self.vulkan.get_swapchain().present(self.vulkan.get_device().borrow().present_queue);
        self.vulkan.get_mut_swapchain().current_frame = (self.vulkan.get_swapchain().current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

fn main() {
    log_panics::init(); // Initialize logging of Rust panics to log files in addition to stdout
    init_log();

    log::info!("Starting Planets!");
    let event_loop = EventLoop::new();
    let app = App::new(&event_loop);
    app.run(event_loop);

    log::info!("Application terminated.");
}

/// Configure logger to write log to console and a separate log file for every execution
fn init_log() {
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
