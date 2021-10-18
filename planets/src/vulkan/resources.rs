use std::cell::RefCell;
use std::rc::Rc;

use ash::vk;
use ash::vk::{Handle, ImageView};

use super::debug;
use super::device::{Device, DeviceMutRef, MAX_FRAMES_IN_FLIGHT};
use super::mem::{AllocatedBuffer, AllocatedBufferMutRef, BufferData};
use crate::vulkan::image::{ImageMutRef, Image};
use crate::vulkan::framebuffer::{FramebufferMutRef, Framebuffer};

pub struct ResourceManager {
    device: DeviceMutRef,
    buffers: Vec<AllocatedBufferMutRef>,
    images: Vec<ImageMutRef>,
    framebuffers: Vec<FramebufferMutRef>,
    pub descriptor_set_manager: DescriptorSetManager,
}

pub type ResourceManagerMutRef = Rc<RefCell<ResourceManager>>;

impl ResourceManager {
    pub fn new(device: &DeviceMutRef) -> ResourceManager {
        let descriptor_set_manager = DescriptorSetManager::new(&device.borrow());
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
        let mut device = self.device.borrow_mut();
        let buffer = Rc::new(RefCell::new(AllocatedBuffer::new_with_size(
            &mut *device,
            size,
            usage,
            mem_props,
        )));
        self.buffers.push(Rc::clone(&buffer));

        debug::Object::label(
            &device,
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
        let mut device = self.device.borrow_mut();
        let buffer = Rc::new(RefCell::new(AllocatedBuffer::new_with_staging(
            &mut *device,
            data,
            usage,
        )));
        self.buffers.push(Rc::clone(&buffer));

        debug::Object::label(
            &device,
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
        let mut device = self.device.borrow_mut();
        let buffer = Rc::new(RefCell::new(AllocatedBuffer::new_host_visible_coherent(
            &mut *device,
            data,
            usage,
        )));
        self.buffers.push(Rc::clone(&buffer));

        debug::Object::label(
            &device,
            vk::ObjectType::BUFFER,
            buffer.borrow().buffer.as_raw(),
            label,
        );

        buffer
    }

    pub fn image_attachment(&mut self, width: u32, height: u32, format: vk::Format, label: &'static str) -> ImageMutRef {
        let image = Rc::new(RefCell::new(Image::new(&self.device, width, height, format, vk::ImageUsageFlags::COLOR_ATTACHMENT, label)));
        self.images.push(Rc::clone(&image));

        image
    }

    pub fn framebuffer(&mut self, width: u32, height: u32, attachments: Vec<ImageView>, render_pass: vk::RenderPass) -> FramebufferMutRef {
        let framebuffer = Rc::new(RefCell::new(Framebuffer::new(&self.device, width, height, attachments, render_pass)));
        self.framebuffers.push(Rc::clone(&framebuffer));

        framebuffer
    }

    pub fn remove_unused(&mut self) {
        let defive_ref = self.device.borrow();
        self.buffers.retain(|buf| {
            if Rc::strong_count(&buf) <= 1 {
                buf.borrow_mut().destroy(&defive_ref);
                return false;
            }

            true
        });

        self.buffers.retain(|img| {
            if Rc::strong_count(&img) <= 1 {
                return false;
            }

            true
        });
    }
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        let device_ref = self.device.borrow();
        for buf in &self.buffers {
            buf.borrow_mut().destroy(&device_ref);
        }
        self.descriptor_set_manager.destroy(&device_ref);
    }
}

pub struct DescriptorSetManager {
    pools: Vec<vk::DescriptorPool>, // TODO: Is Rc needed here?
}

impl DescriptorSetManager {
    fn new(device: &Device) -> DescriptorSetManager {
        DescriptorSetManager { pools: vec![DescriptorSetManager::create_descriptor_pool(device)] }
    }

    pub fn reset_descriptor_pools(&self, device: &Device) {
        for pool in &self.pools {
            unsafe {
                device
                    .logical_device
                    .reset_descriptor_pool(*pool, vk::DescriptorPoolResetFlags::default())
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
        if next_index >= self.pools.len() {
            self.pools.push(DescriptorSetManager::create_descriptor_pool(device));
            log::info!("Allocating additional descriptor pool {}.", next_index);
        }

        let layouts = [*layout];
        let allocate_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool: self.pools[next_index],
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

        return descriptor_set.unwrap()[0];
    }

    fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
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

        let descriptor_pool = unsafe {
            device
                .logical_device
                .create_descriptor_pool(&create_info, None)
                .expect("Failed to create descriptor set")
        };

        descriptor_pool
    }

    fn destroy(&mut self, device: &Device) {
        self.reset_descriptor_pools(device);
        for pool in &self.pools {
            unsafe {
                device.logical_device.destroy_descriptor_pool(*pool, None);
            }
        }
    }
}
