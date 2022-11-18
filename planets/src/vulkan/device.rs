use std::cell::RefCell;
use std::collections::HashSet;
use std::option::Option;
use std::ptr;
use std::rc::Rc;

use ash::vk;

use super::instance::VulkanInstance;
use super::swapchain::{SurfaceDefinition, SwapchainSupportDetails};
use crate::util::helpers;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub type DeviceMutRef = Rc<RefCell<Device>>;

pub struct Device {
    pub entry: ash::Entry,
    pub instance: Rc<VulkanInstance>,
    pub physical_device: vk::PhysicalDevice,
    pub physical_props: vk::PhysicalDeviceProperties,
    pub logical_device: ash::Device,
    pub queue_family_indices: QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub command_pool: Rc<vk::CommandPool>,
    /// TODO: have one pool per frame as described in option #2 here: https://www.reddit.com/r/vulkan/comments/5zwfot/whats_your_command_buffer_allocation_strategy/
    pub command_buffers: Vec<vk::CommandBuffer>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub transfer_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl Device {
    pub fn pick(
        entry: ash::Entry,
        instance: &Rc<VulkanInstance>,
        surface: &SurfaceDefinition,
    ) -> Device {
        let (physical_device, physical_props, queue_indices) =
            Device::find_suitable_devices(&instance.instance, &surface);
        let logical_device =
            Device::create_logical_device(&instance.instance, physical_device, &queue_indices);
        let graphics_queue = Device::get_graphics_queue_handle(&logical_device, &queue_indices);
        let present_queue = Device::get_present_queue_handle(&logical_device, &queue_indices);
        let command_pool = Device::create_command_pool(&logical_device, &queue_indices);
        let command_buffers = Device::create_command_buffers(&logical_device, &command_pool);

        Device {
            entry,
            instance: Rc::clone(&instance),
            physical_device,
            physical_props,
            logical_device,
            queue_family_indices: queue_indices,
            graphics_queue,
            present_queue,
            command_pool,
            command_buffers,
        }
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.logical_device
                .device_wait_idle()
                .expect("Failed to wait for device idle.");
        }
    }

    pub fn recreate(&mut self, surface: &SurfaceDefinition) {
        self.queue_family_indices =
            QueueFamilyIndices::new(&self.instance.instance, self.physical_device, surface);
    }

    pub fn find_memory_type(&self, type_filter: u32, properties: vk::MemoryPropertyFlags) -> u32 {
        let memory_props = unsafe {
            self.instance
                .instance
                .get_physical_device_memory_properties(self.physical_device)
        };

        for (i, mem_type) in memory_props.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) != 0 && (mem_type.property_flags.contains(properties)) {
                return i as u32;
            }
        }

        log::error!("Failed to find memory of matching type.");
        panic!("Failed to find memory of matching type.")
    }

    fn find_suitable_devices(
        instance: &ash::Instance,
        surface: &SurfaceDefinition,
    ) -> (
        vk::PhysicalDevice,
        vk::PhysicalDeviceProperties,
        QueueFamilyIndices,
    ) {
        let devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        for device in devices {
            let queue_indices = QueueFamilyIndices::new(instance, device, surface);
            let swapchain_support = SwapchainSupportDetails::get_for(device, surface);
            if Device::device_suitable(&instance, device, &queue_indices, &swapchain_support) {
                let properties = unsafe { instance.get_physical_device_properties(device) };

                let name = helpers::vulkan_str_to_str(&properties.device_name);
                log::info!("Suitable device: {}", name);

                return (device, properties, queue_indices);
            }
        }

        log::error!("Could not find suitable device!");
        panic!("Could not find suitable device!");
    }

    fn device_suitable(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        queue_indices: &QueueFamilyIndices,
        swapchain_support: &SwapchainSupportDetails,
    ) -> bool {
        // Checking queue family indices
        if !queue_indices.is_complete() {
            return false;
        }

        // Checking swapchain
        if !swapchain_support.adequate() {
            return false;
        }

        // Checking extension
        let required_extensions = helpers::required_device_extension_names();
        let mut required_extension_names = HashSet::new();
        for ext_name in required_extensions {
            required_extension_names.insert(helpers::c_str_ptr_to_str(ext_name));
        }

        let debug_extensions = helpers::required_device_extension_names();
        for ext_name in debug_extensions {
            required_extension_names.insert(helpers::c_str_ptr_to_str(ext_name));
        }

        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .expect("Failed to enumerate device extension properties")
        };
        log::info!("Device extensions:");
        for ext in &available_extensions {
            let ext_name = helpers::vulkan_str_to_str(&ext.extension_name);
            log::info!("{}", ext_name);
            required_extension_names.remove(&ext_name);
        }

        if required_extension_names.len() != 0 {
            return false;
        }

        // for ext in &available_extensions {
        //     println!("{}", helpers::vulkan_str_to_str(&ext.extension_name));
        // }

        true
    }

    fn create_logical_device(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        queue_indices: &QueueFamilyIndices,
    ) -> ash::Device {
        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(queue_indices.graphics_family.unwrap());
        unique_queue_families.insert(queue_indices.present_family.unwrap());

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for queue_family in unique_queue_families {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
            };
            queue_create_infos.push(queue_create_info);
        }
        let device_features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: vk::TRUE,
            ..Default::default()
        };
        let mut device_extensions = helpers::required_device_extension_names();
        let mut debug_extensions = helpers::debug_device_extension_names();
        device_extensions.append(&mut debug_extensions);
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_enabled_features: &device_features,
            enabled_extension_count: device_extensions.len() as u32,
            pp_enabled_extension_names: device_extensions.as_ptr(),
            ..Default::default()
        };

        let logical_device: ash::Device;
        unsafe {
            logical_device = instance
                .create_device(device, &device_create_info, None)
                .expect("Failed to create device");
        }

        logical_device
    }

    fn get_graphics_queue_handle(
        logical_device: &ash::Device,
        indices: &QueueFamilyIndices,
    ) -> vk::Queue {
        let handle = unsafe {
            logical_device.get_device_queue(
                indices
                    .graphics_family
                    .expect("Failed to get graphics queue handle"),
                0 as u32,
            )
        };

        handle
    }

    fn get_present_queue_handle(
        logical_device: &ash::Device,
        indices: &QueueFamilyIndices,
    ) -> vk::Queue {
        let handle = unsafe {
            logical_device.get_device_queue(
                indices
                    .present_family
                    .expect("Failed to get present queue handle"),
                0 as u32,
            )
        };

        handle
    }

    fn create_command_pool(
        device: &ash::Device,
        queue_indices: &QueueFamilyIndices,
    ) -> Rc<vk::CommandPool> {
        let create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            queue_family_index: queue_indices.graphics_family.unwrap(),
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            ..Default::default()
        };

        let command_pool = Rc::new(unsafe {
            device
                .create_command_pool(&create_info, None)
                .expect("Failed to create command pool")
        });

        command_pool
    }

    fn create_command_buffers(
        device: &ash::Device,
        command_pool: &vk::CommandPool,
    ) -> Vec<vk::CommandBuffer> {
        let create_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            command_pool: *command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: MAX_FRAMES_IN_FLIGHT as u32,
            ..Default::default()
        };

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&create_info)
                .expect("Failed to allocate command buffers")
        };

        command_buffers
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .free_command_buffers(*self.command_pool, &self.command_buffers);
            self.logical_device
                .destroy_command_pool(*self.command_pool, None);
            self.logical_device.destroy_device(None);
        }
    }
}

impl QueueFamilyIndices {
    pub fn new(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface: &SurfaceDefinition,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            transfer_family: None,
            compute_family: None,
            present_family: None,
        };

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };

        let mut idx: u32 = 0;
        for family in queue_families {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(idx);
            }
            if family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                indices.transfer_family = Some(idx);
            }
            if family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                indices.compute_family = Some(idx);
            }

            let present_support = unsafe {
                surface
                    .surface_loader
                    .get_physical_device_surface_support(device, idx as u32, surface.surface)
                    .expect("Failed to get physical device surface support")
            };
            if present_support {
                indices.present_family = Some(idx);
            }

            idx += 1;
        }

        indices
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
            && self.transfer_family.is_some()
            && self.compute_family.is_some()
            && self.present_family.is_some()
    }
}
