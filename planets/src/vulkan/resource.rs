use crate::vulkan::device::{DeviceMutRef, Device};
use crate::vulkan::instance::VulkanInstance;
use std::rc::Rc;
use std::cell::RefCell;
use crate::vulkan::swapchain::{Swapchain,SurfaceDefinition};
use crate::util::platforms;

pub struct Resource {
    device: DeviceMutRef,
    instance: Rc<VulkanInstance>,
    swapchain: Swapchain,
    surface: SurfaceDefinition,
}

impl Resource {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { ash::Entry::new().expect("Failed to create ash Entry") };
        let instance = Rc::new(VulkanInstance::new(&entry));
        let surface = Resource::create_surface(&entry, &instance.instance, &window);
        let device = Rc::new(RefCell::new(Device::pick(entry, &instance, &surface)));
        let swapchain = Swapchain::new(&instance.instance, &device, &surface, window.inner_size().width, window.inner_size().height, None);

        Resource {
            device,
            instance,
            swapchain,
            surface,
        }
    }

    fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> SurfaceDefinition {
        let surface = unsafe {
            platforms::create_surface(entry, instance, window).expect("Failed to create surface.")
        };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        SurfaceDefinition { surface_loader, surface }
    }
}