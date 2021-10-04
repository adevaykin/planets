use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use image::io::Reader as ImageReader;

use super::cmd_buffers::SingleTimeCmdBuffer;
use super::debug;
use super::device::{Device, DeviceMutRef};
use super::mem::{AllocatedBufferMutRef, VecBufferData};
use super::resources::ResourceManager;
use super::sampler::Sampler;

pub struct Image {
    device: DeviceMutRef,
    pub image: vk::Image,
    memory: Option<vk::DeviceMemory>,
    layout: vk::ImageLayout,
    format: vk::Format,
    pub views: Vec<vk::ImageView>,
    pub sampler: Sampler,
}

impl Image {
    pub fn new(
        device: &DeviceMutRef,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        label: &str,
    ) -> Image {
        let image = Image::create_image_intern(device, width, height, format, usage, label);

        image
    }

    pub fn from_vk_image(device: &DeviceMutRef, image: vk::Image) -> Image {
        let sampler = Sampler::new(device);

        Image {
            device: Rc::clone(device),
            image,
            memory: None,
            layout: vk::ImageLayout::default(),
            format: vk::Format::R8G8B8A8_SRGB, // TODO: this is a guess, replace with a valid format from existing vk::Image
            views: vec![],
            sampler,
        }
    }

    pub fn from_file(
        device: &DeviceMutRef,
        resource_manager: &mut ResourceManager,
        path: &str,
    ) -> Result<Image, String> {
        let open_file = match ImageReader::open(path) {
            Ok(image) => image,
            Err(_) => return Err(format!("Could not open image file {}", path)),
        };

        let image_data = match open_file.decode() {
            Ok(x) => x,
            Err(_) => return Err(format!("Could not decode image file {}", path)),
        };

        let image_data = image_data.into_rgba8();
        let vec_data_buffer = VecBufferData::new(image_data.as_raw());

        let staging_buffer = ResourceManager::buffer_host_visible_coherent(
            resource_manager,
            &vec_data_buffer,
            vk::BufferUsageFlags::TRANSFER_SRC,
            path,
        );
        staging_buffer
            .borrow()
            .update_data(&*device.borrow(), &vec_data_buffer, 0);

        let usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;
        let mut image = Image::create_image_intern(
            device,
            image_data.width(),
            image_data.height(),
            vk::Format::R8G8B8A8_SRGB,
            usage,
            path,
        );
        image.transition_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        Image::copy_buffer_to_image(
            &*device.borrow(),
            &staging_buffer,
            image.image,
            image_data.width(),
            image_data.height(),
        );
        image.transition_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        let view_cerate_info = vk::ImageViewCreateInfo {
            image: image.image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        image.add_view(view_cerate_info);

        Ok(image)
    }

    pub fn add_view(&mut self, create_info: vk::ImageViewCreateInfo) {
        let image_view = unsafe {
            self.device
                .borrow()
                .logical_device
                .create_image_view(&create_info, None)
                .expect("Failed to create view for swapchaine image")
        };
        self.views.push(image_view);
    }

    pub fn transition_layout(&mut self, new_layout: vk::ImageLayout) {
        let device = self.device.borrow();
        let single_time_cmd_buffer = SingleTimeCmdBuffer::begin(&device);

        let (src_access_mask, dst_access_mask) =
            Image::calculate_access_masks(self.layout, new_layout);
        let (src_stage, dst_stage) = Image::calculate_transition_stages(self.layout, new_layout);
        let aspect_mask = self.calculate_aspect_mask(new_layout);
        let barriers = vec![vk::ImageMemoryBarrier {
            old_layout: self.layout,
            new_layout: new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            src_access_mask: src_access_mask,
            dst_access_mask: dst_access_mask,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: aspect_mask,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        }];

        unsafe {
            self.device.borrow().logical_device.cmd_pipeline_barrier(
                single_time_cmd_buffer.get_cmd_buffer(),
                src_stage,
                dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            );
        }

        self.layout = new_layout;
    }

    fn create_image_intern(
        device: &DeviceMutRef,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        label: &str,
    ) -> Image {
        let initial_layout = vk::ImageLayout::UNDEFINED;

        let create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels: 1,
            array_layers: 1,
            format: format,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout: initial_layout,
            usage: usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let device_ref = device.borrow();
        let image = unsafe {
            device_ref
                .logical_device
                .create_image(&create_info, None)
                .expect("Failed to create image")
        };
        debug::Object::label(&device_ref, vk::ObjectType::IMAGE, image.as_raw(), label);

        let memory = Image::allocate_memory(device, image);
        debug::Object::label(
            &device_ref,
            vk::ObjectType::DEVICE_MEMORY,
            memory.as_raw(),
            label,
        );

        unsafe {
            device_ref
                .logical_device
                .bind_image_memory(image, memory, 0)
                .expect("Failed to bind image memory");
        }

        let sampler = Sampler::new(device);

        Image {
            device: Rc::clone(device),
            image,
            memory: Some(memory),
            layout: initial_layout,
            format: format,
            views: vec![],
            sampler,
        }
    }

    // Returns src and dst access flagsy
    fn calculate_access_masks(
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> (vk::AccessFlags, vk::AccessFlags) {
        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            return (vk::AccessFlags::default(), vk::AccessFlags::TRANSFER_WRITE);
        }

        if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            return (
                vk::AccessFlags::TRANSFER_WRITE,
                vk::AccessFlags::SHADER_READ,
            );
        }

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        {
            return (
                vk::AccessFlags::default(),
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );
        }

        log::error!("Unsupported image layout transition for access mask calculation");
        panic!("Unsupported image layout transition for access mask calculation");
    }

    // Returns src and dst transition stages
    fn calculate_transition_stages(
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> (vk::PipelineStageFlags, vk::PipelineStageFlags) {
        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            return (
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            );
        }

        if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            return (
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            );
        }

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        {
            return (
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            );
        }

        log::error!("Unsupported image layout transition for pipeline stage calculation");
        panic!("Unsupported image layout transition for pipeline stage calculation");
    }

    fn calculate_aspect_mask(&self, new_layout: vk::ImageLayout) -> vk::ImageAspectFlags {
        if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            if Image::has_stencil(self.format) {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            } else {
                vk::ImageAspectFlags::DEPTH
            }
        } else {
            vk::ImageAspectFlags::COLOR
        }
    }

    fn has_stencil(format: vk::Format) -> bool {
        match format {
            vk::Format::D16_UNORM_S8_UINT => true,
            vk::Format::D24_UNORM_S8_UINT => true,
            vk::Format::D32_SFLOAT_S8_UINT => true,
            _ => false,
        }
    }

    fn allocate_memory(device: &DeviceMutRef, image: vk::Image) -> vk::DeviceMemory {
        let device_ref = device.borrow();

        let mem_requirements = unsafe {
            device_ref
                .logical_device
                .get_image_memory_requirements(image)
        };

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: mem_requirements.size,
            memory_type_index: device_ref.find_memory_type(
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ),
            ..Default::default()
        };

        let memory = unsafe {
            device_ref
                .logical_device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate memory for image")
        };

        memory
    }

    fn copy_buffer_to_image(
        device: &Device,
        buffer: &AllocatedBufferMutRef,
        image: vk::Image,
        width: u32,
        height: u32,
    ) {
        let single_time_cmd_buffer = SingleTimeCmdBuffer::begin(device);

        let regions = [vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width: width,
                height: height,
                depth: 1,
            },
        }];

        unsafe {
            device.logical_device.cmd_copy_buffer_to_image(
                single_time_cmd_buffer.get_cmd_buffer(),
                buffer.borrow().buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            for view in &self.views {
                self.device
                    .borrow()
                    .logical_device
                    .destroy_image_view(*view, None);
            }

            if self.memory.is_some() {
                self.device
                    .borrow()
                    .logical_device
                    .destroy_image(self.image, None);
                self.device
                    .borrow()
                    .logical_device
                    .free_memory(self.memory.expect("Failed to free image memory"), None);
            }
        }
    }
}
