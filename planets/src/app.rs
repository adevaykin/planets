use crate::engine::camera::{Camera, CameraMutRef};
use crate::engine::gameloop::{GameLoop, GameLoopMutRef};
use crate::engine::renderer::Renderer;
use crate::engine::viewport::Viewport;
use crate::passes::background::BackgroundPass;
use crate::passes::gameoflife::GameOfLifePass;
use crate::system::serialize::{Loader, Saver};
use crate::util::constants::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::util::helpers::SimpleViewportSize;
use crate::vulkan;
use crate::vulkan::device::MAX_FRAMES_IN_FLIGHT;
use crate::world::gameoflife::GameOfLife;
use crate::world::world::World;
use std::cell::RefCell;
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub const GAME_FIELD_SIZE: usize = 10;

pub struct App {
    gameloop: GameLoopMutRef,
    world: World,
    game_of_life: GameOfLife,
    window: Window,
    vulkan: vulkan::entry::Entry,
    is_paused: bool,
    camera: CameraMutRef,
    renderer: Renderer,
    onpause: bool,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        
        let world = World::new();

        let window = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .unwrap();
        let vulkan = vulkan::entry::Entry::new(&window);
        let gameloop = Rc::new(RefCell::new(GameLoop::new(&mut vulkan.get_resource_manager().borrow_mut())));
        gameloop.borrow_mut().set_max_fps(60);
        // TODO: remove camera instantiation from here
        let camera = Rc::new(RefCell::new(Camera::new(
            &mut vulkan.get_resource_manager().borrow_mut(),
        )));
        let viewport = Rc::new(RefCell::new(Viewport::new(WINDOW_WIDTH, WINDOW_HEIGHT)));
        let background_pass = Box::new(BackgroundPass::new(
            &vulkan.get_device(),
            vulkan.get_resource_manager(),
            &gameloop,
            vulkan.get_shader_manager(),
            &viewport,
            &camera,
        ));

        let game_of_life = GameOfLife::new(&mut vulkan.get_resource_manager().borrow_mut());

        let game_of_life_pass = Box::new(GameOfLifePass::new(
            &vulkan.get_device(),
            vulkan.get_resource_manager(),
            &gameloop,
            vulkan.get_shader_manager(),
            &viewport,
            &camera,
            Rc::clone(&game_of_life.get_gpu_buffer()),
        ));

        let mut renderer = Renderer::new(
            vulkan.get_device(),
            &vulkan.get_resource_manager(),
            &viewport.borrow(),
        );
        renderer.add_pass(background_pass);
        renderer.add_pass(game_of_life_pass);

        App {
            gameloop,
            world,
            game_of_life,
            window,
            vulkan,
            is_paused: false,
            camera,
            renderer,
            onpause: false,
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll; // Continuously poll events even if OS did not provide any

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        self.process_windows_close();
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Destroyed => {
                        self.process_window_destruction();
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::Resized(_) => self.process_resize(),
                    WindowEvent::KeyboardInput { input, .. } => self.process_keyboard_input(&input),
                    WindowEvent::MouseInput {button, state, ..} => {
                        self.process_mouse_input(&state, &button)
                    },
                    _ => {},
                },
                // Window input events were processed - time to start game loop cycle
                Event::MainEventsCleared => self.update_world(),
                Event::Suspended => {
                    self.is_paused = true;
                }
                Event::Resumed => {
                    self.is_paused = false;
                }
                // Window redraw request came in - time to draw
                Event::RedrawRequested { .. } => {
                    if self.is_paused || !self.gameloop.borrow().get_frame_started() {
                        return;
                    }

                    match self.vulkan.get_mut_swapchain().acquire_next_image() {
                        Ok(image_idx) => {
                            self.draw_frame(image_idx);
                        }
                        Err(_) => {
                            let window_size = self.window.inner_size();
                            self.vulkan.recreate_swapchain(
                                None,
                                window_size.width,
                                window_size.height,
                            );
                        }
                    }
                }
                // Drawing ended - finish frame
                Event::RedrawEventsCleared => {
                    self.gameloop.borrow_mut().finish_frame();
                    self.window.set_title(
                        format!("{} | {:.2} FPS", WINDOW_TITLE, self.gameloop.borrow().get_fps()).as_str(),
                    );
                    *control_flow = ControlFlow::WaitUntil(self.gameloop.borrow().get_wait_instant());
                }
                
                _ => (),
            }
        });
    }

    fn toggle_onpause(&mut self){

        self.onpause = !self.onpause;

    }

    fn update_world(&mut self) {
        if !self.gameloop.borrow().should_start_frame() {
            return;
        }

        self.gameloop.borrow_mut().start_frame();

        if self.onpause == false {
            self.game_of_life
            .do_step(self.gameloop.borrow().get_prev_frame_time());

        }
        
        
        self.world.update(self.gameloop.borrow().get_prev_frame_time());

        self.window.request_redraw();
    }

    fn draw_frame(&mut self, image_idx: usize) {
        self.gameloop
            .borrow_mut()
            .update_ubo(&self.vulkan.get_device().borrow());
        let viewport_size = SimpleViewportSize {
            offset_x: 0.0,
            offset_y: 0.0,
            width: self.window.inner_size().width as f32,
            height: self.window.inner_size().height as f32,
        };
        self.camera
            .borrow_mut()
            .update(&self.vulkan.get_device().borrow(), &viewport_size);

        self.game_of_life.update(&self.vulkan.get_device().borrow());

        self.vulkan.start_frame(image_idx);
        self.renderer.render(image_idx);
        self.renderer.blit_result(
            image_idx,
            &mut self.vulkan.get_mut_swapchain().images[image_idx],
        );

        //self.draw_list.borrow_mut().cull(image_idx, &self.scene.borrow_mut());

        self.vulkan
            .get_swapchain()
            .submit(&self.vulkan.get_device().borrow().command_buffers[image_idx]);
        //self.draw_list.borrow_mut().end_frame(image_idx);
        self.vulkan
            .get_swapchain()
            .present(self.vulkan.get_device().borrow().present_queue);
        self.vulkan.get_mut_swapchain().current_frame =
            (self.vulkan.get_swapchain().current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn process_resize(&mut self) {
        if self.window.inner_size().width == 0 || self.window.inner_size().height == 0 {
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
            }
            KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::L),
                state: ElementState::Released,
                ..
            } => {
                let loader = Loader::new();
                self.world = loader.load();
            }
            KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::P),
                state: ElementState::Released,
                ..
            } => {
                self.toggle_onpause();
            }
            _ => {}
        }
    }

    fn process_mouse_input(&self, state: &ElementState, button: &MouseButton){
        if *button == MouseButton::Left && *state == ElementState::Pressed {
            log::info!("Left button");
        }
    }
}
