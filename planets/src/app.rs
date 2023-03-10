use crate::engine::camera::{Camera, CameraMutRef};
use crate::engine::gameloop::{GameLoop, GameLoopMutRef};
use crate::engine::renderer::Renderer;
use crate::engine::viewport::{Viewport, ViewportMutRef};
use crate::engine::passes::gbuffer::GBufferPass;
use crate::vulkan;
use crate::vulkan::device::MAX_FRAMES_IN_FLIGHT;
use std::cell::RefCell;
use std::rc::Rc;
use ash::vk;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::engine::passes::background::BackgroundPass;
use crate::engine::renderpass::RenderPass;
use crate::engine::scene::builder::build_scene;
use crate::engine::scene::graph::{SceneGraph, SceneGraphMutRef};
use crate::engine::window::Window;
use crate::util::constants::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::world::loader::ModelLoader;

pub struct App {
    gameloop: GameLoopMutRef,
    window: Window,
    vulkan: vulkan::entry::Entry,
    is_paused: bool,
    camera: CameraMutRef,
    renderer: Renderer,
    viewport: ViewportMutRef,
    scene: SceneGraphMutRef,
    gbuffer_pass: GBufferPass,
    background_pass: BackgroundPass,
    onpause: bool,
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let os_window = Window::create_os_window(event_loop);
        let vulkan = vulkan::entry::Entry::new(&os_window);
        let window = Window::new(&vulkan.get_instance().instance, vulkan.get_device(), vulkan.get_surface(), os_window);

        let gameloop = Rc::new(RefCell::new(GameLoop::new(&mut vulkan.get_resource_manager().borrow_mut())));
        gameloop.borrow_mut().set_max_fps(60);
        // TODO: remove camera instantiation from here
        let camera = Rc::new(RefCell::new(Camera::new(
            &mut vulkan.get_resource_manager().borrow_mut(),
        )));
        let viewport = Rc::new(RefCell::new(Viewport::new(WINDOW_WIDTH, WINDOW_HEIGHT)));
        let background_pass = BackgroundPass::new(
            vulkan.get_device(),
            vulkan.get_resource_manager(),
            &gameloop,
            &mut vulkan.get_shader_manager().borrow_mut(),
            &viewport,
            &camera,
        );

        let model_loader = Rc::new(RefCell::new(ModelLoader::new(
            vulkan.get_resource_manager(),
            vulkan.get_texture_manager()
        )));
        let scene = SceneGraph::new_mut_ref(vulkan.get_device(), vulkan.get_resource_manager());
        build_scene(&mut scene.borrow_mut(), &mut model_loader.borrow_mut());

        let gbuffer_pass = GBufferPass::new(
            vulkan.get_device(),
            vulkan.get_resource_manager(),
            &gameloop,
            &mut vulkan.get_shader_manager().borrow_mut(),
            &viewport,
            &camera,
            &scene
        );

        let renderer = Renderer::new(
            vulkan.get_device(),
        );

        App {
            gameloop,
            window,
            vulkan,
            is_paused: false,
            camera,
            renderer,
            viewport,
            scene,
            background_pass,
            gbuffer_pass,
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

                    if let Some(swapchain) = self.window.get_mut_swapchain() {
                        match swapchain.acquire_next_image() {
                            Ok(image_idx) => {
                                self.draw_frame(image_idx);
                            }
                            Err(_) => {
                                self.window.recreate_swapchain(&self.vulkan.get_instance().instance, self.vulkan.get_device(), self.vulkan.get_surface());
                            }
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

        if !self.onpause {
            // noop yet
        }

        self.window.request_redraw();
    }

    fn draw_frame(&mut self, image_idx: usize) {
        self.vulkan.get_device().borrow_mut().set_image_idx(image_idx);
        self.gameloop
            .borrow_mut()
            .update_ubo(&self.vulkan.get_device().borrow());
        let window_size = self.window.get_size();
        self.camera
            .borrow_mut()
            .update(&self.vulkan.get_device().borrow(), window_size.x, window_size.y);
        self.scene.borrow_mut().update(&self.vulkan.get_device().borrow(), &self.gameloop.borrow());
        self.scene.borrow().get_light_manager().borrow_mut().update(&self.vulkan.get_device().borrow());

        // Game logic update here

        self.vulkan.start_frame();

        let scene_drawables = self.scene.borrow_mut().cull();
        self.scene.borrow_mut().get_draw_list().borrow_mut().add_drawables(scene_drawables);

        self.renderer.begin_frame();
        self.vulkan.get_texture_manager().borrow_mut().upload_pending();

        let gbuffer_outputs = self.gbuffer_pass.run(self.vulkan.get_device().borrow().get_command_buffer(), vec![]);
        let background_outputs = self.background_pass.run(self.vulkan.get_device().borrow().get_command_buffer(), gbuffer_outputs);

        if let Some(swapchain) = self.window.get_mut_swapchain() {
            let device = self.vulkan.get_device().borrow();
            device.blit_result(&mut background_outputs[0].borrow_mut(), &mut swapchain.images[image_idx]);
            device.transition_layout(&mut swapchain.images[image_idx], vk::ImageLayout::PRESENT_SRC_KHR);
        }

        self.renderer.end_frame();

        if let Some(swapchain) = self.window.get_swapchain() {
            swapchain.submit(self.vulkan.get_device().borrow().get_command_buffer());
        }

        self.scene.borrow_mut().get_draw_list().borrow_mut().end_frame();

        let present_queue = self.vulkan.get_device().borrow().present_queue;
        if let Some(swapchain) = self.window.get_mut_swapchain() {
            swapchain.present(present_queue);
        }
    }

    fn process_resize(&mut self) {
        let window_size = self.window.get_size();
        if window_size.x == 0 || window_size.y == 0 {
            self.is_paused = true;
            return;
        } else {
            self.is_paused = false;
        }

        self.window.destroy_swapchain();
        self.vulkan.initialize_for_window(self.window.get_os_window());
        self.window.recreate_swapchain(&self.vulkan.get_instance().instance, self.vulkan.get_device(), self.vulkan.get_surface());
        self.viewport.borrow_mut().update(window_size.x, window_size.y);
    }

    fn process_windows_close(&mut self) {
        log::info!("Exit requested by window close request.");
        self.process_window_destruction();
    }

    fn process_window_destruction(&mut self) {
        log::info!("Exit on window destruction.");
        self.is_paused = true;
        self.vulkan.get_device().borrow().wait_idle();
    }

    fn process_keyboard_input(&mut self, keyboard_input_event: &KeyboardInput) {
        if let KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::P),
                state: ElementState::Released,
                ..
            } = keyboard_input_event {
            self.toggle_onpause();
        }
    }

    fn process_mouse_input(&self, state: &ElementState, button: &MouseButton){
        if *button == MouseButton::Left && *state == ElementState::Pressed {
            log::info!("Left button");
        }
    }
}
