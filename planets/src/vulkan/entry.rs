use crate::util::platforms;
use crate::vulkan::device::{Device, DeviceMutRef};
use crate::vulkan::instance::VulkanInstance;
use crate::vulkan::resources::{ResourceManager, ResourceManagerMutRef};
use crate::vulkan::shader::{ShaderManager, ShaderManagerMutRef};
use crate::vulkan::swapchain::{SurfaceDefinition, Swapchain};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Entry {
    device: DeviceMutRef,
    instance: Rc<VulkanInstance>,
    swapchain: Swapchain,
    surface: SurfaceDefinition,
    resource_manager: ResourceManagerMutRef,
    shader_manager: ShaderManagerMutRef,
}

impl Entry {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { ash::Entry::new().expect("Failed to create ash Entry") };
        let instance = Rc::new(VulkanInstance::new(&entry));
        let surface = Entry::create_surface(&entry, &instance.instance, &window);
        let device = Rc::new(RefCell::new(Device::pick(entry, &instance, &surface)));
        let swapchain = Swapchain::new(
            &instance.instance,
            &device,
            &surface,
            window.inner_size().width,
            window.inner_size().height,
            None,
        );
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new(&device)));
        let shader_manager = Rc::new(RefCell::new(ShaderManager::new(&device)));

        Entry {
            device,
            instance,
            swapchain,
            surface,
            resource_manager,
            shader_manager,
        }
    }

    pub fn start_frame(&mut self, image_idx: usize) {
        self.resource_manager.borrow_mut().remove_unused();
        self.resource_manager
            .borrow_mut()
            .descriptor_set_manager
            .reset_descriptor_pools(&self.device.borrow(), image_idx);
    }

    pub fn get_device(&self) -> &DeviceMutRef {
        &self.device
    }

    pub fn get_swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn get_mut_swapchain(&mut self) -> &mut Swapchain {
        &mut self.swapchain
    }

    pub fn get_resource_manager(&self) -> &ResourceManagerMutRef {
        &self.resource_manager
    }

    pub fn get_shader_manager(&self) -> &ShaderManagerMutRef {
        &self.shader_manager
    }

    pub fn initialize_for_window(&mut self, window: &winit::window::Window) {
        self.device.borrow().wait_idle();
        self.surface = Entry::create_surface(&self.device.borrow().entry, &self.instance.instance, &window);
        self.device.borrow_mut().recreate(&self.surface);
        self.recreate_swapchain(None, window.inner_size().width, window.inner_size().height);
    }

    pub fn recreate_swapchain(&mut self, surface: Option<&SurfaceDefinition>, width: u32, height: u32) {
        self.swapchain = Swapchain::new(
            &self.instance.instance,
            &self.device,
            surface.unwrap_or(&self.surface),
            width,
            height,
            Some(self.swapchain.swapchain),
        );
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> SurfaceDefinition {
        let surface = unsafe {
            platforms::create_surface(entry, instance, window).expect("Failed to create surface.")
        };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        SurfaceDefinition {
            surface_loader,
            surface,
        }
    }
}
