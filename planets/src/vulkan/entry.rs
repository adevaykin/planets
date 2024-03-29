use crate::vulkan::device::{Device, DeviceMutRef};
use crate::vulkan::instance::VulkanInstance;
use crate::vulkan::resources::manager::{ResourceManager, ResourceManagerMutRef};
use crate::vulkan::resources::objects::{ObjectDescriptions, ObjectDescriptionsMutRef};
use crate::vulkan::shader::{ShaderManager, ShaderManagerMutRef};
use crate::vulkan::swapchain::{SurfaceDefinition};
use std::cell::RefCell;
use std::rc::Rc;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use crate::engine::textures::{TextureManager, TextureManagerMutRef};
use crate::engine::viewport::Viewport;
use crate::util::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};

pub struct Entry {
    ash_entry: ash::Entry,
    device: DeviceMutRef,
    instance: Rc<VulkanInstance>,
    surface: SurfaceDefinition,
    resource_manager: ResourceManagerMutRef,
    shader_manager: ShaderManagerMutRef,
    texture_manager: TextureManagerMutRef,
    object_descriptions: ObjectDescriptionsMutRef,
}

impl Entry {
    pub fn new(os_window: &winit::window::Window) -> Self {
        let ash_entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan library") };
        let instance = Rc::new(VulkanInstance::new(&ash_entry));
        let surface = Self::create_surface(&ash_entry, &instance.instance, os_window);
        let device = Rc::new(RefCell::new(Device::pick(ash_entry.clone(), &instance, &surface)));
        let temp_viewport = Rc::new(RefCell::new(Viewport::new(WINDOW_WIDTH, WINDOW_HEIGHT)));
        let resource_manager = Rc::new(RefCell::new(ResourceManager::new(&device, &temp_viewport)));
        let shader_manager = Rc::new(RefCell::new(ShaderManager::new(&device)));
        let texture_manager = Rc::new(RefCell::new(TextureManager::new(&device, &resource_manager)));
        let object_descriptions = Rc::new(RefCell::new(ObjectDescriptions::new()));

        Entry {
            ash_entry,
            device,
            instance,
            surface,
            resource_manager,
            shader_manager,
            texture_manager,
            object_descriptions,
        }
    }

    pub fn get_instance(&self) -> &VulkanInstance {
        &self.instance
    }

    pub fn get_surface(&self) -> &SurfaceDefinition {
        &self.surface
    }

    pub fn start_frame(&mut self) {
        let mut resource_manager = self.resource_manager.borrow_mut();
        resource_manager.on_frame_start();
        self.object_descriptions.borrow_mut().update(&mut resource_manager);
    }

    pub fn get_device(&self) -> &DeviceMutRef {
        &self.device
    }

    pub fn get_resource_manager(&self) -> &ResourceManagerMutRef {
        &self.resource_manager
    }

    pub fn get_object_descriptions(&self) -> &ObjectDescriptionsMutRef {
        &self.object_descriptions
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
            device_ref.wait_idle();
            self.surface = Entry::create_surface(
                &self.ash_entry,
                &self.instance.instance,
                window,
            );
        }
        self.device.borrow_mut().recreate(&self.surface);
    }

    pub fn create_surface(
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
