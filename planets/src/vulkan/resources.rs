use alloc::rc::Weak;
use std::cell::RefCell;
use std::rc::Rc;

use ash::vk;
use ash::vk::{Handle, ImageView};
use crate::vulkan::device::DeviceMutRef;

use super::debug;
use super::device::{Device, MAX_FRAMES_IN_FLIGHT};
use super::mem::{AllocatedBuffer, AllocatedBufferMutRef, BufferData};
use crate::vulkan::framebuffer::{Framebuffer, FramebufferMutRef};
use crate::vulkan::img::image::{Image, ImageMutRef};

pub type ResourceManagerMutRef = Rc<RefCell<ResourceManager>>;

// TODO: make it member of Device
pub struct ResourceManager {
    device: DeviceMutRef,
    buffers: Vec<AllocatedBufferMutRef>,
    images: Vec<ImageMutRef>,
    framebuffers: Vec<FramebufferMutRef>,
    pub descriptor_set_manager: DescriptorSetManager,
}

impl ResourceManager {
    pub fn new(device: &DeviceMutRef) -> ResourceManager {
        let descriptor_set_manager = DescriptorSetManager::new(device);
        ResourceManager {
            device: Rc::clone(device),
            buffers: vec![],
            images: vec![],
            framebuffers: vec![],
            descriptor_set_manager,
        }
    }

    pub fn buffer_with_size(
        &mut self,
        size: u64,
        usage: vk::BufferUsageFlags,
        mem_props: vk::MemoryPropertyFlags,
        label: &str,
    ) -> AllocatedBufferMutRef {
        let buffer = Rc::new(RefCell::new(AllocatedBuffer::new_with_size(
            &self.device,
            size,
            usage,
            mem_props,
        )));
        self.buffers.push(Rc::clone(&buffer));

        debug::Object::label(
            &self.device.borrow(),
            vk::ObjectType::BUFFER,
            buffer.borrow().buffer.as_raw(),
            label,
        );

        buffer
    }

    // TODO: should it return MutRef or maybe not?
    pub fn buffer_with_staging(
        &mut self,
        data: &impl BufferData,
        usage: vk::BufferUsageFlags,
        label: &str,
    ) -> AllocatedBufferMutRef {
        let buffer = Rc::new(RefCell::new(AllocatedBuffer::new_with_staging(
            &self.device,
            data,
            usage,
        )));
        self.buffers.push(Rc::clone(&buffer));

        debug::Object::label(
            &self.device.borrow(),
            vk::ObjectType::BUFFER,
            buffer.borrow().buffer.as_raw(),
            label,
        );

        buffer
    }

    pub fn buffer_host_visible_coherent(
        &mut self,
        data: &impl BufferData,
        usage: vk::BufferUsageFlags,
        label: &str,
    ) -> AllocatedBufferMutRef {
        let buffer = Rc::new(RefCell::new(AllocatedBuffer::new_host_visible_coherent(
            &self.device,
            data,
            usage,
        )));
        self.buffers.push(Rc::clone(&buffer));

        debug::Object::label(
            &self.device.borrow(),
            vk::ObjectType::BUFFER,
            buffer.borrow().buffer.as_raw(),
            label,
        );

        buffer
    }

    pub fn image(
        &mut self,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        label: &'static str,
    ) -> ImageMutRef {
        let image = Rc::new(RefCell::new(Image::new(
            &self.device,
            width,
            height,
            format,
            usage,
        )));

        self.images.push(Rc::clone(&image));

        debug::Object::label(
            &self.device.borrow(),
            vk::ObjectType::IMAGE,
            image.borrow().get_image().as_raw(),
            label,
        );

        image
    }

    pub fn framebuffer(
        &mut self,
        width: u32,
        height: u32,
        attachments: &Vec<ImageView>,
        render_pass: vk::RenderPass,
        label: &str,
    ) -> FramebufferMutRef {
        let framebuffer = Rc::new(RefCell::new(Framebuffer::new(
            &self.device,
            width,
            height,
            attachments,
            render_pass,
        )));
        self.framebuffers.push(Rc::clone(&framebuffer));

        debug::Object::label(
            &self.device.borrow(),
            vk::ObjectType::FRAMEBUFFER,
            framebuffer.borrow().framebuffer.as_raw(),
            label,
        );

        framebuffer
    }

    pub fn remove_unused(&mut self) {
        self.buffers.retain(|buf| {
            Rc::strong_count(buf) > 1
        });

        self.buffers.retain(|img| {
            if Rc::strong_count(img) <= 1 {
                return false;
            }

            true
        });
    }
}

pub struct DescriptorSetManager {
    device: Weak<RefCell<Device>>,
    pools: [Vec<Rc<vk::DescriptorPool>>; MAX_FRAMES_IN_FLIGHT],
    pool_in_use: usize,
}

impl DescriptorSetManager {
    fn new(device: &DeviceMutRef) -> DescriptorSetManager {
        let pools = [
            vec![DescriptorSetManager::create_descriptor_pool(&device.borrow())],
            vec![DescriptorSetManager::create_descriptor_pool(&device.borrow())],
        ];

        DescriptorSetManager {
            device: Rc::downgrade(device),
            pools,
            pool_in_use: 0,
        }
    }

    pub fn reset_descriptor_pools(&mut self, device: &Device, image_idx: usize) {
        self.pool_in_use = image_idx;
        for pool in &self.pools[self.pool_in_use] {
            unsafe {
                device
                    .logical_device
                    .reset_descriptor_pool(**pool, vk::DescriptorPoolResetFlags::default())
                    .expect("Failed to reset descriptor set.");
            }
        }
    }

    pub fn allocate_descriptor_set(
        &mut self,
        device: &Device,
        layout: &ash::vk::DescriptorSetLayout,
    ) -> vk::DescriptorSet {
        self.try_allocate_descriptor_set(device, layout, 0)
    }

    fn try_allocate_descriptor_set(
        &mut self,
        device: &Device,
        layout: &ash::vk::DescriptorSetLayout,
        next_index: usize,
    ) -> vk::DescriptorSet {
        let frame_pools = &mut self.pools[self.pool_in_use];
        if next_index >= frame_pools.len() {
            frame_pools.push(DescriptorSetManager::create_descriptor_pool(device));
            log::info!("Allocating additional descriptor pool {}.", next_index);
        }

        let pool = &frame_pools[next_index];
        let layouts = [*layout];
        let allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: **pool, // TODO: remove Device::descriptor_pool
            descriptor_set_count: 1,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };

        let descriptor_set = unsafe {
            device
                .logical_device
                .allocate_descriptor_sets(&allocate_info)
        };
        if descriptor_set.is_err() {
            return self.try_allocate_descriptor_set(device, layout, next_index + 1);
        }

        descriptor_set.unwrap()[0]
    }

    fn create_descriptor_pool(device: &Device) -> Rc<vk::DescriptorPool> {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 10,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 10,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 10,
            },
        ];

        const MAX_DESCRIPTOR_SETS: u32 = 1024;
        let create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            max_sets: MAX_DESCRIPTOR_SETS,
            ..Default::default()
        };

        Rc::new(unsafe {
            device
                .logical_device
                .create_descriptor_pool(&create_info, None)
                .expect("Failed to create descriptor set")
        })
    }
}

impl Drop for DescriptorSetManager {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            let device_ref = device.borrow();

            for frame in 0..MAX_FRAMES_IN_FLIGHT {
                for pool in &self.pools[frame] {
                    unsafe {
                        device_ref.logical_device.destroy_descriptor_pool(**pool, None);
                    }
                }
            }
        }
    }
}
