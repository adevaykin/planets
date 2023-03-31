use std::rc::Rc;

use ash::vk;
use crate::vulkan::fence::Fence;
use crate::vulkan::img::image::Image;
use crate::vulkan::semaphore::Semaphore;

use super::device::{DeviceMutRef, MAX_FRAMES_IN_FLIGHT};

pub struct SurfaceDefinition {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
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
    pub images: Vec<Image>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_available_sems: Vec<Semaphore>,
    pub render_finished_sems: Vec<Semaphore>,
    pub in_flight_fences: Vec<Fence>,
    in_flight_images: Vec<Option<vk::Fence>>,
}

impl SwapchainSupportDetails {
    pub fn get_for(
        physical_device: vk::PhysicalDevice,
        surface: &SurfaceDefinition,
    ) -> SwapchainSupportDetails {
        unsafe {
            let capabilities = surface
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface.surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface.surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface
                .surface_loader
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
            if fmt.format == vk::Format::B8G8R8A8_SRGB
                && fmt.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *fmt;
            }
        }

        *self.formats.first().unwrap()
    }

    pub fn choose_present_mode(&self) -> vk::PresentModeKHR {
        for mode in &self.present_modes {
            if *mode == vk::PresentModeKHR::MAILBOX {
                return *mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    pub fn choose_extent(&self, width: u32, height: u32) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            self.capabilities.current_extent
        } else {
            use num::clamp;
            vk::Extent2D {
                width: clamp(
                    width,
                    self.capabilities.min_image_extent.width,
                    self.capabilities.max_image_extent.width,
                ),
                height: clamp(
                    height,
                    self.capabilities.min_image_extent.height,
                    self.capabilities.max_image_extent.height,
                ),
            }
        }
    }
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &DeviceMutRef,
        surface: &SurfaceDefinition,
        width: u32,
        height: u32,
        old_swapchain: &Option<Swapchain>,
    ) -> Swapchain {
        let device_ref = device.borrow();
        let swapchain_support =
            SwapchainSupportDetails::get_for(device_ref.physical_device, surface);
        let extent = swapchain_support.choose_extent(width, height);
        let format = swapchain_support.choose_format();
        let present_mode = swapchain_support.choose_present_mode();

        let image_count =
            if swapchain_support.capabilities.max_image_count >= MAX_FRAMES_IN_FLIGHT as u32 {
                MAX_FRAMES_IN_FLIGHT as u32
            } else {
                swapchain_support.capabilities.min_image_count + 1
            };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if device_ref.queue_family_indices.graphics_family
                != device_ref.queue_family_indices.present_family
            {
                (
                    vk::SharingMode::EXCLUSIVE,
                    2,
                    vec![
                        device_ref.queue_family_indices.graphics_family.unwrap(),
                        device_ref.queue_family_indices.present_family.unwrap(),
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let old_swapchain = if let Some(old_swapchain) = old_swapchain {
            old_swapchain.swapchain
        } else {
            vk::SwapchainKHR::null()
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface.surface,
            min_image_count: image_count,
            image_color_space: format.color_space,
            image_format: format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain,
            image_array_layers: 1,
            ..Default::default()
        };

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(instance, &device.borrow().logical_device);
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
        for (i,image) in swapchain_images.iter().enumerate() {
            let wrapped = Image::from_vk_image(
                device,
                *image,
                width,
                height,
                vk::Format::R8G8B8A8_SRGB,
                format!("Swapchain-{}", i).as_str()
            ); // TODO: format is a guess
            wrapped_images.push(wrapped);
        }



        let mut image_available_sems = vec![];
        let mut render_finished_sems = vec![];
        let mut in_flight_fences = vec![];
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_sems.push(Semaphore::new(&device, format!("ImageAvailable{}", i).as_str()));
            render_finished_sems.push(Semaphore::new(&device, format!("RenderFinished{}", i).as_str()));
            in_flight_fences.push(Fence::new(&device, vk::FenceCreateFlags::SIGNALED, format!("InFlight{}", i).as_str()));
        }

        let in_flight_images = vec![None; wrapped_images.len()];

        let mut swapchain = Swapchain {
            device: Rc::clone(device),
            current_frame: 0,
            loader: swapchain_loader,
            swapchain,
            format: format.format,
            extent,
            images: wrapped_images,
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
        let fences = [self.in_flight_fences[self.current_frame].get_fence()];
        unsafe {
            device_ref
                .logical_device
                .wait_for_fences(&fences, true, u64::MAX)
                .expect("Failed to wait for in-flight fences");
        }

        let image_idx = match unsafe {
            self.loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_sems[self.current_frame].get_semaphore(),
                vk::Fence::null(),
            )
        } {
            Err(err) => {
                return Err(err);
            }
            Ok((idx, _)) => idx,
        };

        if let Some(fence) = self.in_flight_images[image_idx as usize] {
            let fences = [fence];
            unsafe {
                device_ref
                    .logical_device
                    .wait_for_fences(&fences, true, u64::MAX)
                    .expect("Failed to wait for image available fences");
            }
        }

        self.in_flight_images[image_idx as usize] = Some(self.in_flight_fences[self.current_frame].get_fence());

        Ok(image_idx as usize)
    }

    pub fn reset_inflight_fence(&self) {
        let fences = [self.in_flight_fences[self.device.borrow().get_image_idx()].get_fence()];
        unsafe {
            self.device
                .borrow()
                .logical_device
                .reset_fences(&fences)
                .expect("Failed to reset fences");
        }
    }

    pub fn submit(&self, command_buffer: vk::CommandBuffer) {
        let device_ref = self.device.borrow();
        let wait_semaphores = [self.image_available_sems[self.current_frame].get_semaphore()];
        let signal_semaphores = [self.render_finished_sems[self.current_frame].get_semaphore()];
        let pipeline_stage_flags = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let cmd_buffers = [command_buffer];
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

        self.reset_inflight_fence();
        unsafe {
            device_ref
                .logical_device
                .queue_submit(
                    device_ref.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame].get_fence(),
                )
                .expect("Failed to submit queue");
        }
    }

    pub fn present(&self, present_queue: vk::Queue) {
        let wait_semaphores = [self.render_finished_sems[self.current_frame].get_semaphore()];
        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            swapchain_count: swapchains.len() as u32,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &(self.current_frame as u32),
            ..Default::default()
        };

        if unsafe { self.loader.queue_present(present_queue, &present_info) }.is_err() {
            log::error!("QueuePresent returned error.");
        }
    }

    fn create_swapchain_views(swapchain: &mut Swapchain) {
        for image in &mut swapchain.images {
            if let Err(msg) = image.add_get_view(swapchain.format) {
                log::error!("{}", msg);
            }
        }
    }

    pub fn destroy(&self) {
        unsafe {
            self.loader.destroy_swapchain(self.swapchain, None);
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.destroy();
    }
}
