use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use crate::util::constants::{WINDOW_HEIGHT, WINDOW_TITLE, WINDOW_WIDTH};
use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::swapchain::{SurfaceDefinition, Swapchain};

pub struct Window {
    os_window: winit::window::Window,
    swapchain: Option<Swapchain>,
}

impl Window {
    pub fn new(instance: &ash::Instance, device: &DeviceMutRef, surface: &SurfaceDefinition, os_window: winit::window::Window) -> Self {
        let swapchain = Swapchain::new(
            instance,
            device,
            surface,
            os_window.inner_size().width,
            os_window.inner_size().height,
            &None,
        );

        Window {
            os_window,
            swapchain: Some(swapchain),
        }
    }

    pub fn create_os_window(event_loop: &EventLoop<()>) -> winit::window::Window {
        WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .unwrap()
    }

    pub fn get_mut_swapchain(&mut self) -> &mut Option<Swapchain> {
        &mut self.swapchain
    }

    pub fn get_swapchain(&self) -> &Option<Swapchain> {
        &self.swapchain
    }

    pub fn get_os_window(&self) -> &winit::window::Window {
        &self.os_window
    }

    pub fn destroy_swapchain(&mut self) {
        self.swapchain = None;
    }

    pub fn recreate_swapchain(
        &mut self,
        ash_instance: &ash::Instance,
        device: &DeviceMutRef,
        surface: &SurfaceDefinition,
    ) {
        let window_size = self.os_window.inner_size();

        self.swapchain = Some(Swapchain::new(
            ash_instance,
            device,
            surface,
            window_size.width,
            window_size.height,
            &self.swapchain,
        ));
    }

    pub fn set_title(&self, title: &str) {
        self.os_window.set_title(title);
    }

    pub fn request_redraw(&self) {
        self.os_window.request_redraw();
    }

    pub fn get_size(&self) -> cgmath::Vector2::<u32> {
        cgmath::Vector2::new(self.os_window.inner_size().width, self.os_window.inner_size().height)
    }
}