use std::cell::RefCell;
use std::collections::HashSet;
use std::option::Option;
use std::ptr;
use std::rc::Rc;

use ash::vk;

use super::instance::VulkanInstance;
use super::swapchain::{SurfaceDefinition, SwapchainSupportDetails};
use crate::util::helpers;
use crate::vulkan::{extensions};
use crate::vulkan::img::image::{Image, ImageAccess};
use crate::vulkan::rt::pipeline::RtPipeline;

pub const MAX_FRAMES_IN_FLIGHT: usize = 3;

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
    pub rt_pipeline: RtPipeline,
    image_idx: usize,
    pub command_pool: Rc<vk::CommandPool>,
    /// TODO: have one pool per frame as described in option #2 here: https://www.reddit.com/r/vulkan/comments/5zwfot/whats_your_command_buffer_allocation_strategy/
    command_buffers: Vec<vk::CommandBuffer>,
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
            Device::find_suitable_devices(&instance.instance, surface);
        let logical_device =
            Device::create_logical_device(&instance.instance, physical_device, &queue_indices);
        let graphics_queue = Device::get_graphics_queue_handle(&logical_device, &queue_indices);
        let present_queue = Device::get_present_queue_handle(&logical_device, &queue_indices);
        let rt_pipeline = RtPipeline::new(&instance.instance, &physical_device, &logical_device);
        let command_pool = Device::create_command_pool(&logical_device, &queue_indices);
        let command_buffers = Device::create_command_buffers(&logical_device, &command_pool);

        Device {
            entry,
            instance: Rc::clone(instance),
            physical_device,
            physical_props,
            logical_device,
            queue_family_indices: queue_indices,
            graphics_queue,
            present_queue,
            rt_pipeline,
            image_idx: 0,
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

    pub fn set_image_idx(&mut self, idx: usize) {
        self.image_idx = idx;
    }

    pub fn get_physical_device_mem_props(&self) -> vk::PhysicalDeviceMemoryProperties {
        unsafe {
            self.instance
                .instance
                .get_physical_device_memory_properties(self.physical_device)
        }
    }

    pub fn get_buffer_device_address(&self, buffer: vk::Buffer) -> u64 {
        let buffer_device_address_info = vk::BufferDeviceAddressInfo::builder()
            .buffer(buffer)
            .build();

        unsafe { self.logical_device.get_buffer_device_address(&buffer_device_address_info) }
    }

    pub fn find_memory_type(&self, type_filter: u32, properties: vk::MemoryPropertyFlags) -> Result<u32,&str> {
        let memory_props = unsafe {
            self.instance
                .instance
                .get_physical_device_memory_properties(self.physical_device)
        };

        for (i, mem_type) in memory_props.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) != 0 && (mem_type.property_flags.contains(properties)) {
                return Ok(i as u32);
            }
        }

        Err("Failed to find memory of matching type.")
    }

    pub fn blit_result(&self, src_image: &mut Image, dst_image: &mut Image) {
        let cmd_buffer = self.get_command_buffer();
        let src_offsets = [
            vk::Offset3D { x: 0, y: 0, z: 0 },
            vk::Offset3D {
                x: src_image.get_width() as i32,
                y: src_image.get_height() as i32,
                z: 1,
            },
        ];
        let dst_offsets = src_offsets;
        let regions = [vk::ImageBlit {
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_offsets,
            dst_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            dst_offsets,
        }];

        let src_image_access = ImageAccess {
            new_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            src_stage: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage: vk::PipelineStageFlags::TRANSFER,
            src_access: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dst_access: vk::AccessFlags::TRANSFER_READ,
        };
        let src_img = src_image.access_image(self, &src_image_access);

        let dst_image_access = ImageAccess {
            new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            src_stage: vk::PipelineStageFlags::TRANSFER,
            dst_stage: vk::PipelineStageFlags::TRANSFER,
            src_access: vk::AccessFlags::TRANSFER_READ,
            dst_access: vk::AccessFlags::TRANSFER_WRITE,
        };
        let dst_img = dst_image.access_image(self, &dst_image_access);

        unsafe {
            self.logical_device.cmd_blit_image(
                cmd_buffer,
                src_img,
                src_image.get_layout(),
                dst_img,
                dst_image.get_layout(),
                &regions,
                vk::Filter::NEAREST,
            );
        }
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
            let properties = unsafe { instance.get_physical_device_properties(device) };
            let name = helpers::vulkan_str_to_str(&properties.device_name);
            log::info!("Inspecting device {}", name);
            let queue_indices = QueueFamilyIndices::new(instance, device, surface);
            let swapchain_support = SwapchainSupportDetails::get_for(device, surface);
            if Device::device_suitable(instance, device, &queue_indices, &swapchain_support) {
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
            log::info!("\tQueue indices incomplete.");
            return false;
        }

        // Checking swapchain
        if !swapchain_support.adequate() {
            log::info!("\tSwapchain support inadequate.");
            return false;
        }

        // Checking extension
        let required_extensions = extensions::required_device_extension_names();
        let mut required_extension_names = HashSet::new();
        for ext_name in required_extensions {
            required_extension_names.insert(helpers::c_str_ptr_to_str(ext_name));
        }

        let debug_extensions = extensions::debug_device_extension_names();
        for ext_name in debug_extensions {
            required_extension_names.insert(helpers::c_str_ptr_to_str(ext_name));
        }

        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .expect("Failed to enumerate device extension properties")
        };
        log::debug!("Available device extensions:");
        for ext in &available_extensions {
            let ext_name = helpers::vulkan_str_to_str(&ext.extension_name);
            log::debug!("{}", ext_name);
            required_extension_names.remove(&ext_name);
        }

        if !required_extension_names.is_empty() {
            log::info!("\tNot all required extensions are supported:");
            for ext in &required_extension_names {
                log::info!("\t\t{}", ext);
            }
            return false;
        }

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
        let features1 = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: vk::TRUE,
            ..Default::default()
        };
        let mut device_extensions = extensions::required_device_extension_names();
        let mut debug_extensions = extensions::debug_device_extension_names();
        device_extensions.append(&mut debug_extensions);

        // Query and request ray tracing features
        let mut features12 = vk::PhysicalDeviceVulkan12Features::builder()
            .buffer_device_address(true)
            .vulkan_memory_model(true)
            .build();
        let mut as_feature = vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
        let mut raytracing_pipeline_feature = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default();
        let mut ray_query_feature = vk::PhysicalDeviceRayQueryFeaturesKHR::default();
        let mut features2 = vk::PhysicalDeviceFeatures2KHR::builder()
            .push_next(&mut features12)
            .push_next(&mut as_feature)
            .push_next(&mut raytracing_pipeline_feature)
            .push_next(&mut ray_query_feature)
            .features(features1)
            .build();
        unsafe {
            instance.get_physical_device_features2(device, &mut features2)
        };

        let device_create_info = vk::DeviceCreateInfo::builder()
            .push_next(&mut features2)
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .build();

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
        unsafe {
            logical_device.get_device_queue(
                indices
                    .graphics_family
                    .expect("Failed to get graphics queue handle"),
                0,
            )
        }
    }

    fn get_present_queue_handle(
        logical_device: &ash::Device,
        indices: &QueueFamilyIndices,
    ) -> vk::Queue {
        unsafe {
            logical_device.get_device_queue(
                indices
                    .present_family
                    .expect("Failed to get present queue handle"),
                0,
            )
        }
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

        Rc::new(unsafe {
            device
                .create_command_pool(&create_info, None)
                .expect("Failed to create command pool")
        })
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

        unsafe {
            device
                .allocate_command_buffers(&create_info)
                .expect("Failed to allocate command buffers")
        }
    }

    pub fn get_image_idx(&self) -> usize {
        self.image_idx
    }

    pub fn get_prev_image_idx(&self) -> usize {
        ((self.image_idx as i32 + MAX_FRAMES_IN_FLIGHT as i32) % MAX_FRAMES_IN_FLIGHT as i32) as usize
    }

    pub(crate) fn get_command_buffer(&self) -> vk::CommandBuffer {
        self.command_buffers[self.image_idx]
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

        for (idx,family) in queue_families.iter().enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(idx as u32);
            }
            if family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                indices.transfer_family = Some(idx as u32);
            }
            if family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                indices.compute_family = Some(idx as u32);
            }

            let present_support = unsafe {
                surface
                    .surface_loader
                    .get_physical_device_surface_support(device, idx as u32, surface.surface)
                    .expect("Failed to get physical device surface support")
            };
            if present_support {
                indices.present_family = Some(idx as u32);
            }
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
