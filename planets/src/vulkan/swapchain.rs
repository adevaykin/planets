use std::rc::Rc;

use ash::vk;

use crate::util::helpers;
use super::image;
use super::device::{DeviceMutRef,MAX_FRAMES_IN_FLIGHT};

pub struct SurfaceDefinition {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR
}

impl Drop for SurfaceDefinition {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

pub struct SwapchainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

pub struct Swapchain {
    device: DeviceMutRef,
    pub current_frame: usize,
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<image::Image>,
    pub depth_images: Vec<image::Image>,
    pub format: vk::Format,
    pub depth_format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_available_sems: Vec<vk::Semaphore>,
    pub render_finished_sems: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    in_flight_images: Vec<Option<vk::Fence>>,
}

impl SwapchainSupportDetails {
    pub fn get_for(physical_device: vk::PhysicalDevice, surface: &SurfaceDefinition) -> SwapchainSupportDetails {
        unsafe {
            let capabilities = surface.surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface.surface_loader
                .get_physical_device_surface_formats(physical_device, surface.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface.surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface.surface)
                .expect("Failed to query for surface present mode.");

            SwapchainSupportDetails {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    pub fn adequate(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }

    pub fn choose_format(&self) -> vk::SurfaceFormatKHR {
        for fmt in &self.formats {
            if fmt.format == vk::Format::B8G8R8A8_SRGB && fmt.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return fmt.clone();
            }
        }

        self.formats.first().unwrap().clone()
    }

    pub fn choose_depth_format(&self, instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> vk::Format {
        let candidates = vec![
            vk::Format::D24_UNORM_S8_UINT,
            vk::Format::D32_SFLOAT_S8_UINT,
        ];

        for format in candidates {
            let props = unsafe { instance.get_physical_device_format_properties(physical_device, format) };
                
            if props.optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT) {
                return format;
            }
        }

        log::error!("Could not find required depth format!");
        panic!("Could not find required depth format!");
    }

    pub fn choose_present_mode(&self) -> vk::PresentModeKHR {
        for mode in &self.present_modes {
            if *mode == vk::PresentModeKHR::MAILBOX {
                return mode.clone();
            }
        }

        vk::PresentModeKHR::FIFO
    }

    pub fn choose_extent(&self, width: u32, height: u32) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::max_value() {
            self.capabilities.current_extent
        } else {
            use num::clamp;
            vk::Extent2D {
                width: clamp(
                    width as u32,
                    self.capabilities.min_image_extent.width,
                    self.capabilities.max_image_extent.width,
                ),
                height: clamp(
                    height as u32,
                    self.capabilities.min_image_extent.height,
                    self.capabilities.max_image_extent.height,
                ),
            }
        }
    }
}

impl Swapchain {
    pub fn new(instance: &ash::Instance, device: &DeviceMutRef, surface: &SurfaceDefinition, width: u32, height: u32,
        old_swapchain: Option<vk::SwapchainKHR>)
        -> Swapchain {
        let devicqe_ref = device.borrow();
        let swapchain_support = SwapchainSupportDetails::get_for(devicqe_ref.physical_device, &surface);
        let extent = swapchain_support.choose_extent(width, height);
        let format = swapchain_support.choose_format();
        let depth_format = swapchain_support.choose_depth_format(instance, devicqe_ref.physical_device);
        let present_mode = swapchain_support.choose_present_mode();

        let image_count = if swapchain_support.capabilities.max_image_count >= MAX_FRAMES_IN_FLIGHT as u32 {
            MAX_FRAMES_IN_FLIGHT as u32
        } else {
            swapchain_support.capabilities.min_image_count + 1
        };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if devicqe_ref.queue_family_indices.graphics_family != devicqe_ref.queue_family_indices.present_family {
                (
                    vk::SharingMode::EXCLUSIVE,
                    2,
                    vec![
                        devicqe_ref.queue_family_indices.graphics_family.unwrap(),
                        devicqe_ref.queue_family_indices.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface.surface,
            min_image_count: image_count,
            image_color_space: format.color_space,
            image_format: format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: old_swapchain.unwrap_or(vk::SwapchainKHR::null()),
            image_array_layers: 1,
            ..Default::default()
        };

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, &device.borrow().logical_device);
        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create Swapchain!")
        };

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to get Swapchain Images.")
        };

        let mut wrapped_images = vec![];
        for image in swapchain_images {
            let wrapped = image::Image::from_vk_image(&device, image);
            wrapped_images.push(wrapped);
        }

        let mut wrapped_depth_images = vec![];
        for _ in 0..wrapped_images.len() {
            let mut image = image::Image::new(device, extent.width, extent.height, depth_format, vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, "DepthAttachment");
            let view_cerate_info = vk::ImageViewCreateInfo {
                image: image.image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: depth_format,
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                },
                ..Default::default()
            };
            image.add_view(view_cerate_info);
            wrapped_depth_images.push(image);
        }

        let sem_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_sems = vec![];
        let mut render_finished_sems = vec![];
        let mut in_flight_fences = vec![];
        for _ in 0..super::device::MAX_FRAMES_IN_FLIGHT {
            let image_available_sem = unsafe {
                devicqe_ref.logical_device.create_semaphore(&sem_create_info, None).expect("Failed to create image available semaphore")
            };
            let render_finished_sem = unsafe {
                devicqe_ref.logical_device.create_semaphore(&sem_create_info, None).expect("Failed to create render finished semaphore")
            };
            let in_flight_fence = unsafe {
                devicqe_ref.logical_device.create_fence(&fence_create_info, None).expect("Failed to create in-flight fence")
            };

            image_available_sems.push(image_available_sem);
            render_finished_sems.push(render_finished_sem);
            in_flight_fences.push(in_flight_fence);
        }

        let in_flight_images = vec![None; wrapped_images.len()];

        let mut swapchain = Swapchain {
            device: Rc::clone(&device),
            current_frame: 0,
            loader: swapchain_loader,
            swapchain,
            format: format.format,
            depth_format: depth_format,
            extent,
            images: wrapped_images,
            depth_images: wrapped_depth_images,
            image_available_sems,
            render_finished_sems,
            in_flight_fences,
            in_flight_images,
        };

        Swapchain::create_swapchain_views(&mut swapchain);

        swapchain
    }

    pub fn acquire_next_image(&mut self) -> Result<usize, vk::Result> {
        let device_ref = self.device.borrow();
        let fences = [self.in_flight_fences[self.current_frame]];
        unsafe {
            device_ref.logical_device.wait_for_fences(&fences, true, std::u64::MAX).expect("Failed to wait for in-flight fences");
        }

        let image_idx = match unsafe { self.loader.acquire_next_image(self.swapchain, std::u64::MAX, self.image_available_sems[self.current_frame], vk::Fence::null())} {
            Err(err) => { return Err(err); },
            Ok((idx, _)) => idx
        };

        match self.in_flight_images[image_idx as usize] {
            Some(fence) => {
                let fences = [fence];
                unsafe { device_ref.logical_device.wait_for_fences(&fences, true, std::u64::MAX).expect("Failed to wait for image available fences"); }
            }
            _ => ()
        }

        self.in_flight_images[image_idx as usize] = Some(self.in_flight_fences[self.current_frame]);

        Ok(image_idx as usize)
    }

    pub fn reset_inflight_fence(&self) {
        let fences = [self.in_flight_fences[self.current_frame]];
        unsafe {
            self.device.borrow().logical_device.reset_fences(&fences).expect("Failed to reset fences");
        }
    }

    pub fn submit(&self, command_buffer: &vk::CommandBuffer) {
        let wait_semaphores = [self.image_available_sems[self.current_frame as usize]];
        let signal_semaphores = [self.render_finished_sems[self.current_frame as usize]];
        let pipeline_stage_flags = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let cmd_buffers = [*command_buffer];
        let submit_infos = [vk::SubmitInfo {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: pipeline_stage_flags.as_ptr(),
            command_buffer_count: cmd_buffers.len() as u32,
            p_command_buffers: cmd_buffers.as_ptr(),
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        }];

        let device_ref = self.device.borrow();
        self.reset_inflight_fence();
        unsafe {
            device_ref.logical_device.queue_submit(
                device_ref.graphics_queue,
                &submit_infos,
                self.in_flight_fences[self.current_frame as usize]
            ).expect("Failed to submit queue");
        }
    }

    pub fn present(&self, present_queue: vk::Queue) {
        let wait_semaphores = [self.render_finished_sems[self.current_frame]];
        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: swapchains.len() as u32,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &(self.current_frame as u32),
            ..Default::default()
        };

        match unsafe { self.loader.queue_present(present_queue, &present_info) } {
            Err(_) => { log::warn!("QueuePresent returned error."); },
            Ok(_) => {}
        }
    }

    fn create_swapchain_views(swapchain: &mut Swapchain) {
        for image in &mut swapchain.images {
            let view_create_info = vk::ImageViewCreateInfo {
                image: image.image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: swapchain.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1
                },
                ..Default::default()
            };
            image.add_view(view_create_info);
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        let device_ref = self.device.borrow();
        unsafe {
            self.loader.destroy_swapchain(self.swapchain, None);
            for sem in &self.image_available_sems {
                device_ref.logical_device.destroy_semaphore(*sem, None);
            }
            for sem in &self.render_finished_sems {
                device_ref.logical_device.destroy_semaphore(*sem, None);
            }
            for fence in &self.in_flight_fences {
                device_ref.logical_device.destroy_fence(*fence, None);
            }
        }
    }
}