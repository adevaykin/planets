use crate::gameloop::GameLoop;
use crate::vulkan;
use crate::engine::timer::{TimerMutRef, Timer};
use crate::engine::camera::{CameraMutRef, Camera};
use crate::engine::renderer::Renderer;
use winit::event_loop::{EventLoop, ControlFlow};
use crate::world::world::World;
use winit::window::{WindowBuilder, Window};
use winit::dpi::PhysicalSize;
use crate::util::constants::{WINDOW_WIDTH, WINDOW_HEIGHT, WINDOW_TITLE};
use std::cell::RefCell;
use std::rc::Rc;
use crate::engine::viewport::Viewport;
use crate::passes::background::BackgroundPass;
use winit::event::{WindowEvent, Event, KeyboardInput, VirtualKeyCode, ElementState};
use crate::system::serialize::{Saver, Loader};
use crate::util::helpers::SimpleViewportSize;
use crate::vulkan::device::MAX_FRAMES_IN_FLIGHT;

pub struct App {
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
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let mut gameloop = GameLoop::new();
        gameloop.set_max_fps(2);

        let world = World::new();

        let window = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop).unwrap();
        let vulkan = vulkan::entry::Entry::new(&window);
        // TODO: remove camera instantiation from here
        let camera = Rc::new(RefCell::new(Camera::new(&mut vulkan.get_resource_manager().borrow_mut())));
        let timer = Rc::new(RefCell::new(Timer::new(&mut vulkan.get_resource_manager().borrow_mut())));
        let viewport = Rc::new(RefCell::new(Viewport::new(WINDOW_WIDTH, WINDOW_HEIGHT)));
        let background_pass = Box::new(BackgroundPass::new(&vulkan.get_device(), vulkan.get_resource_manager(), &timer, vulkan.get_shader_manager(), &viewport, &camera, "Background"));
        //let game_of_life_pass = Box::new(GameOfLifePass::new());

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

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll; // Continuously poll events even if OS did not provide any

            match event {
                Event::WindowEvent {
                    event, ..
                } => match event {
                    WindowEvent::CloseRequested => {
                        self.process_windows_close();
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::Destroyed => {
                        self.process_window_destruction();
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::Resized(_) => self.process_resize(),
                    WindowEvent::KeyboardInput { input, .. } => self.process_keyboard_input(&input),
                    _ => {}
                },
                // Window input events were processed - time to start game loop cycle
                Event::MainEventsCleared => self.update_world(),
                Event::Suspended => {
                    self.is_paused = true;
                },
                Event::Resumed => {
                    self.is_paused = false;
                }
                // Window redraw request came in - time to draw
                Event::RedrawRequested {
                    ..
                } => {
                    if self.is_paused || !self.gameloop.get_frame_started() {
                        return;
                    }

                    match self.vulkan.get_mut_swapchain().acquire_next_image() {
                        Ok(image_idx) => {
                            self.draw_frame(image_idx);
                        },
                        Err(_) => {
                            let window_size = self.window.inner_size();
                            self.vulkan.recreate_swapchain(None, window_size.width, window_size.height);
                            // TODO: recreate render passes here?
                        }
                    }
                },
                // Drawing ended - finish frame
                Event::RedrawEventsCleared => {
                    self.gameloop.finish_frame();
                    self.window.set_title(format!("{} | {:.2} FPS", WINDOW_TITLE, self.gameloop.get_fps()).as_str());
                    *control_flow = ControlFlow::WaitUntil(self.gameloop.get_wait_instant());
                },
                _ => (),
            }
        });
    }

    fn update_world(&mut self) {
        if !self.gameloop.should_start_frame() {
            return;
        }

        self.gameloop.start_frame();

        log::info!("Frame {} started.", self.gameloop.get_frame_num());
        self.world.update(self.gameloop.get_prev_frame_time());
        log::info!("World status: {}", self.world.get_description_string());

        self.window.request_redraw();
    }

    fn draw_frame(&mut self, image_idx: usize) {
        self.timer.borrow_mut().update(&self.gameloop, &self.vulkan.get_device().borrow());
        let viewport_size = SimpleViewportSize {
            offset_x: 0.0,
            offset_y: 0.0,
            width: self.window.inner_size().width as f32,
            height: self.window.inner_size().height as f32
        };
        self.camera.borrow_mut().update(&self.vulkan.get_device().borrow(), &viewport_size);

        self.vulkan.start_frame(image_idx);
        self.renderer.render(image_idx);
        self.renderer.blit_result(image_idx, &mut self.vulkan.get_mut_swapchain().images[image_idx]);

        //self.draw_list.borrow_mut().cull(image_idx, &self.scene.borrow_mut());

        self.vulkan.get_swapchain().submit(&self.vulkan.get_device().borrow().command_buffers[image_idx]);
        //self.draw_list.borrow_mut().end_frame(image_idx);
        self.vulkan.get_swapchain().present(self.vulkan.get_device().borrow().present_queue);
        self.vulkan.get_mut_swapchain().current_frame = (self.vulkan.get_swapchain().current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn process_resize(&mut self) {
        if self.window.inner_size().width == 0  || self.window.inner_size().height == 0 {
            self.is_paused = true;
            return;
        } else {
            self.is_paused = false;
        }

        self.vulkan.initialize_for_window(&self.window);
    }

    fn process_windows_close(&mut self) {
        log::info!("Exit requested by window close request.");
        self.is_paused = true;
        self.vulkan.get_device().borrow().wait_idle();
    }

    fn process_window_destruction(&mut self) {
        log::info!("Exit on window destruction.");
        self.is_paused = true;
        self.vulkan.get_device().borrow().wait_idle();
    }

    fn process_keyboard_input(&mut self, keyboard_input_event: &KeyboardInput) {
        match keyboard_input_event {
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
        }
    }
}