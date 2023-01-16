use crate::vulkan::device::{Device, DeviceMutRef};
use crate::vulkan::instance::VulkanInstance;
use crate::vulkan::resources::{ResourceManager, ResourceManagerMutRef};
use crate::vulkan::shader::{ShaderManager, ShaderManagerMutRef};
use crate::vulkan::swapchain::{SurfaceDefinition, Swapchain};
use std::cell::RefCell;
use std::rc::Rc;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use crate::engine::textures::{TextureManager, TextureManagerMutRef};

pub struct Entry {
    device: DeviceMutRef,
    instance: Rc<VulkanInstance>,
    swapchain: Option<Swapchain>,
    surface: SurfaceDefinition,
    resource_manager: ResourceManagerMutRef,
    shader_manager: ShaderManagerMutRef,
    texture_manager: TextureManagerMutRef,
}

impl Entry {
    pub fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan library") };
        let instance = Rc::new(VulkanInstance::new(&entry));
        let surface = Entry::create_surface(&entry, &instance.instance, &window);
        let device = Rc::new(RefCell::new(Device::pick(entry, &instance, &surface)));
        let swapchain = Swapchain::new(
            &instance.instance,
            &device,
            &surface,
            window.inner_size().width,
            window.inner_size().height,
            &None,
        );
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new(&device)));
        let shader_manager = Rc::new(RefCell::new(ShaderManager::new(&device)));
        let texture_manager = Rc::new(RefCell::new(TextureManager::new(&device, &resource_manager)));

        Entry {
            device,
            instance,
            swapchain: Some(swapchain),
            surface,
            resource_manager,
            shader_manager,
            texture_manager,
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

    pub fn get_swapchain(&self) -> &Option<Swapchain> {
        &self.swapchain
    }

    pub fn get_mut_swapchain(&mut self) -> &mut Option<Swapchain> {
        &mut self.swapchain
    }

    pub fn get_resource_manager(&self) -> &ResourceManagerMutRef {
        &self.resource_manager
    }

    pub fn get_shader_manager(&self) -> &ShaderManagerMutRef {
        &self.shader_manager
    }

    pub fn get_texture_manager(&self) -> &TextureManagerMutRef {
        &self.texture_manager
    }

    pub fn initialize_for_window(&mut self, window: &winit::window::Window) {
        {
            let device_ref = self.device.borrow();
            //self.swapchain.submit(device_ref.get_command_buffer());
            device_ref.wait_idle();
            self.swapchain = None;
            self.surface = Entry::create_surface(
                &self.device.borrow().entry,
                &self.instance.instance,
                window,
            );
        }
        self.device.borrow_mut().recreate(&self.surface);
        self.recreate_swapchain(None, window.inner_size().width, window.inner_size().height);
    }

    pub fn recreate_swapchain(
        &mut self,
        surface: Option<&SurfaceDefinition>,
        width: u32,
        height: u32,
    ) {
        self.swapchain = Some(Swapchain::new(
            &self.instance.instance,
            &self.device,
            surface.unwrap_or(&self.surface),
            width,
            height,
            &self.swapchain,
        ));
    }

    fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> SurfaceDefinition {
        unsafe {
            let surface = ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            ).expect("Failed to create Vulkan surface.");
            let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

            SurfaceDefinition {
                surface_loader,
                surface,
            }
        }
    }
}
