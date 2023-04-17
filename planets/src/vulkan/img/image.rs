use alloc::rc::Weak;
use std::rc::Rc;

use ash::vk;
use ash::vk::{Handle};

use image::io::Reader as ImageReader;

use std::cell::RefCell;
use std::collections::HashMap;
use image::DynamicImage;
use crate::vulkan::debug;
use crate::vulkan::debug::DebugResource;
use crate::vulkan::device::{Device, DeviceMutRef};
use crate::vulkan::img::sampler::Sampler;
use crate::vulkan::mem::{AllocatedBuffer, Memory, VecBufferData};
use crate::vulkan::resources::manager::ResourceManager;

pub type ImageMutRef = Rc<RefCell<Image>>;

pub struct ImageAccess {
    pub new_layout: vk::ImageLayout,
    pub src_stage: vk::PipelineStageFlags,
    pub dst_stage: vk::PipelineStageFlags,
    pub src_access: vk::AccessFlags,
    pub dst_access: vk::AccessFlags,
}

pub struct MemoryBarrier {
    image: vk::Image,
    aspect: vk::ImageAspectFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_stage: vk::PipelineStageFlags,
    src_access: vk::AccessFlags,
    dst_stage: vk::PipelineStageFlags,
    dst_access: vk::AccessFlags
}

impl MemoryBarrier {
    fn record(&self, device: &Device) {
        let barriers = [vk::ImageMemoryBarrier::builder()
            .old_layout(self.old_layout)
            .new_layout(self.new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .src_access_mask(self.src_access)
            .dst_access_mask(self.dst_access)
            .image(self.image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: self.aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build()];

        unsafe {
            device.logical_device.cmd_pipeline_barrier(
                device.get_command_buffer(),
                self.src_stage,
                self.dst_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            );
        }
    }
}

pub struct Image {
    data: Option<DynamicImage>,
    device: Weak<RefCell<Device>>,
    image: vk::Image,
    memory: Option<Memory>,
    layout: vk::ImageLayout,
    #[allow(dead_code)]
    format: vk::Format,
    width: u32,
    height: u32,
    pub views: HashMap<vk::Format, vk::ImageView>,
    pub sampler: Sampler,
    label: String,
}

impl Image {
    pub fn new(
        device: &DeviceMutRef,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        label: &'static str,
    ) -> Image {
        Image::create_image_intern(device, width, height, format, usage, label)
    }

    pub fn from_vk_image(
        device: &DeviceMutRef,
        image: vk::Image,
        width: u32,
        height: u32,
        format: vk::Format,
        label: &str,
    ) -> Image {
        let sampler = Sampler::new(device);

        Image {
            data: None,
            device: Rc::downgrade(device),
            image,
            memory: None,
            layout: vk::ImageLayout::default(),
            format,
            width,
            height,
            views: HashMap::new(),
            sampler,
            label: String::from(label),
        }
    }

    pub fn from_file(
        device: &DeviceMutRef,
        path: &str,
    ) -> Result<Image, String> {
        let open_file = match ImageReader::open(path) {
            Ok(image) => image,
            Err(_) => return Err(format!("Could not open image file {}", path)),
        };

        let usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED;
        let image_data = match open_file.decode() {
            Ok(x) => x,
            Err(_) => return Err(format!("Could not decode image file {}", path)),
        };

        let mut image = Image::create_image_intern(
            device,
            image_data.width(),
            image_data.height(),
            vk::Format::R8G8B8A8_SRGB,
            usage,
            path,
        );

        image.data = Some(image_data);

        Ok(image)
    }

    // Get view with the default defined during image creation
    pub fn get_view(&self) -> vk::ImageView {
        *self.views.get(&self.format).unwrap()
    }

    pub fn access_view(&mut self, device: &Device, barrier_params: &ImageAccess, format: vk::Format) -> Result<vk::ImageView,String> {
        match self.views.get(&format) {
            Some(view) => Ok(*view),
            None => {
                let image = self.access_image(device, barrier_params);
                let view_create_info = vk::ImageViewCreateInfo {
                    image,
                    view_type: vk::ImageViewType::TYPE_2D,
                    format,
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: Image::aspect_mask_from_format(format),
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    ..Default::default()
                };

                if let Some(device) = self.device.upgrade() {
                    match unsafe {
                        device
                            .borrow()
                            .logical_device
                            .create_image_view(&view_create_info, None)
                    } {
                        Ok(image_view) => {
                            self.views.insert(format, image_view);
                            self.access_view(&device.borrow(), barrier_params, format)
                        },
                        Err(_) => {
                            Err(format!("Failed to create view for image {}", self.label))
                        }
                    }
                } else {
                    Err(format!("Could not upgrade device weak to create image view for {}", self.label))
                }
            }
        }
    }

    pub fn access_image(&mut self, device: &Device, barrier_params: &ImageAccess) -> vk::Image {
        let barrier = MemoryBarrier {
            image: self.image,
            aspect: Image::aspect_mask_from_format(self.format),
            old_layout: self.layout,
            new_layout: barrier_params.new_layout,
            src_stage: barrier_params.src_stage,
            src_access: barrier_params.src_access,
            dst_stage: barrier_params.dst_stage,
            dst_access: barrier_params.dst_access,
        };

        barrier.record(device);
        self.layout = barrier_params.new_layout;

        self.image
    }

    pub fn get_layout(&self) -> vk::ImageLayout {
        self.layout
    }

    pub fn set_layout(&mut self, layout: vk::ImageLayout) {
        self.layout = layout;
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
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
            format,
            tiling: vk::ImageTiling::OPTIMAL,
            initial_layout,
            usage,
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

        // TODO: mem allocation should be done via ResourceManager or Device?
        let memory = match Image::allocate_memory(&device_ref, image, "Image") {
            Ok(mem) => {
                debug::Object::label(&device_ref, &mem);

                unsafe {
                    device_ref
                        .logical_device
                        .bind_image_memory(image, mem.get_memory(), 0)
                        .expect("Failed to bind image memory");
                }

                Some(mem)
            },
            Err(msg) => {
                log::error!("{}", msg);
                None
            }
        };

        Image {
            data: None,
            device: Rc::downgrade(device),
            image,
            memory,
            layout: initial_layout,
            format,
            width,
            height,
            views: HashMap::new(),
            sampler: Sampler::new(device),
            label: String::from(label),
        }
    }

    fn aspect_mask_from_format(format: vk::Format) -> vk::ImageAspectFlags {
        match format {
            vk::Format::D16_UNORM => vk::ImageAspectFlags::DEPTH,
            vk::Format::D16_UNORM_S8_UINT => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            vk::Format::D24_UNORM_S8_UINT => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            vk::Format::D32_SFLOAT => vk::ImageAspectFlags::DEPTH,
            vk::Format::D32_SFLOAT_S8_UINT => vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            _ => vk::ImageAspectFlags::COLOR,
        }
    }

    // TODO: this should be part of ResourceManager or Device
    fn allocate_memory(device: &Device, image: vk::Image, label: &str) -> Result<Memory,&'static str> {
        let mem_requirements = unsafe {
            device
                .logical_device
                .get_image_memory_requirements(image)
        };

        let memory_type_index = device.find_memory_type(
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ).expect("Could not find required device memory type");

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: mem_requirements.size,
            memory_type_index,
            ..Default::default()
        };

        unsafe {
            match device
                .logical_device
                .allocate_memory(&allocate_info, None)
            {
                Ok(res) => Ok(Memory::new(res, label)),
                Err(_) => Err("Failed to allocate Vulkan memory")
            }

        }
    }

    fn copy_buffer_to_image(
        device: &Device,
        buffer: &AllocatedBuffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) {
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
                width,
                height,
                depth: 1,
            },
        }];

        unsafe {
            device.logical_device.cmd_copy_buffer_to_image(
                device.get_command_buffer(),
                buffer.buffer,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
        }
    }

    pub fn upload(&mut self, device: &Device, resource_manager: &mut ResourceManager) -> Result<(), ()> {
        if let Some(data) = self.data.take() {
            let image_data = data.into_rgba8();
            let vec_data_buffer = VecBufferData::new(image_data.as_raw());

            let staging_buffer = ResourceManager::buffer_host_visible_coherent(
                resource_manager,
                &vec_data_buffer,
                vk::BufferUsageFlags::TRANSFER_SRC,
                "Staging",
            );
            let staging_borrow_ref = staging_buffer.borrow();
            staging_borrow_ref.update_data(device, &vec_data_buffer, 0);

            let barrier_params = ImageAccess {
                new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                src_stage: vk::PipelineStageFlags::TOP_OF_PIPE,
                src_access: vk::AccessFlags::default(),
                dst_stage: vk::PipelineStageFlags::TRANSFER,
                dst_access: vk::AccessFlags::TRANSFER_WRITE,
            };
            let image = self.access_image(device, &barrier_params);
            Image::copy_buffer_to_image(
                device,
                &staging_borrow_ref,
                image,
                image_data.width(),
                image_data.height(),
            );

            Ok(())
        } else {
            Err(())
        }
    }
}

impl DebugResource for Image {
    fn get_type(&self) -> vk::ObjectType {
        vk::ObjectType::IMAGE
    }

    fn get_handle(&self) -> u64 {
        self.image.as_raw()
    }

    fn get_label(&self) -> &String {
        &self.label
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                for view in self.views.values() {
                    device
                        .borrow()
                        .logical_device
                        .destroy_image_view(*view, None);
                }

                if let Some(memory) = &self.memory {
                    device
                        .borrow()
                        .logical_device
                        .destroy_image(self.image, None);
                    device
                        .borrow()
                        .logical_device
                        .free_memory(memory.get_memory(), None);
                }
            }
        } else {
            log::error!("Could not upgrade device weak to destroy image and views.");
        }
    }
}
