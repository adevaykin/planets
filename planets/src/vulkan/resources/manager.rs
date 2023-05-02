use alloc::rc::Weak;
use std::cell::RefCell;
use std::rc::Rc;

use ash::vk;
use ash::vk::{ImageView};
use crate::engine::viewport::ViewportMutRef;
use crate::vulkan::device::DeviceMutRef;

use crate::vulkan::debug;
use crate::vulkan::device::{Device, MAX_FRAMES_IN_FLIGHT};
use crate::vulkan::mem::{AllocatedBuffer, AllocatedBufferMutRef, BufferData};
use crate::vulkan::framebuffer::{Framebuffer, FramebufferMutRef};
use crate::vulkan::img::image::{Image, ImageMutRef};

pub type ResourceManagerMutRef = Rc<RefCell<ResourceManager>>;

pub enum Attachment {
    FixedSize(ImageMutRef),
    FramebufferSize(ImageMutRef, f32) // Image and framebuffer size ratio
}

pub enum AttachmentSize {
    Fixed(u32, u32),
    Relative(f32),
}

pub struct ResourceManager {
    device: DeviceMutRef,
    viewport: ViewportMutRef,
    buffers: Vec<Vec<AllocatedBufferMutRef>>,
    images: Vec<Vec<Attachment>>,
    framebuffers: Vec<Vec<FramebufferMutRef>>,
    pub descriptor_set_manager: DescriptorSetManager,
}

impl ResourceManager {
    pub fn new(device: &DeviceMutRef, viewport: &ViewportMutRef) -> ResourceManager {
        let descriptor_set_manager = DescriptorSetManager::new(device);
        ResourceManager {
            device: Rc::clone(device),
            viewport: Rc::clone(viewport),
            buffers: vec![vec![]; MAX_FRAMES_IN_FLIGHT],
            images: vec![vec![]; MAX_FRAMES_IN_FLIGHT],
            framebuffers: vec![vec![]; MAX_FRAMES_IN_FLIGHT],
            descriptor_set_manager,
        }
    }

    pub fn on_frame_start(&mut self) {
        self.remove_unused();
        self
            .descriptor_set_manager
            .reset_descriptor_pools(&self.device.borrow());
    }

    #[allow(dead_code)]
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
            label,
        )));
        self.buffers[self.device.borrow().get_image_idx()].push(Rc::clone(&buffer));

        debug::Object::label(&self.device.borrow(),&*buffer.borrow());

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
            label,
        )));
        self.buffers[self.device.borrow().get_image_idx()].push(Rc::clone(&buffer));
        debug::Object::label(&self.device.borrow(),&*buffer.borrow());

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
            label,
        )));
        self.buffers[self.device.borrow().get_image_idx()].push(Rc::clone(&buffer));
        debug::Object::label(&self.device.borrow(), &*buffer.borrow());

        buffer
    }

    pub fn attachment(
        &mut self,
        size: AttachmentSize,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        label: &'static str,
    ) -> ImageMutRef {
        let viewport_ref = self.viewport.borrow();
        let (width, height) = match size {
            AttachmentSize::Fixed(w,h) => (w, h),
            AttachmentSize::Relative(ratio) => (viewport_ref.width, viewport_ref.height),
        };
        let image = Rc::new(RefCell::new(Image::new(
            &self.device,
            width,
            height,
            format,
            usage,
            label,
        )));

        self.images[self.device.borrow().get_image_idx()].push(Rc::clone(&image));
        debug::Object::label(&self.device.borrow(), &*image.borrow());

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
            label
        )));
        self.framebuffers[self.device.borrow().get_image_idx()].push(Rc::clone(&framebuffer));

        debug::Object::label(&self.device.borrow(), &*framebuffer.borrow());

        framebuffer
    }

    pub fn remove_unused(&mut self) {
        let frame_idx = self.device.borrow().get_prev_image_idx();
        self.buffers[frame_idx].retain(|buf| {
            Rc::strong_count(buf) > 1
        });

        self.images[frame_idx].retain(|buf| {
            Rc::strong_count(buf) > 1
        });

        self.framebuffers[frame_idx].retain(|buf| {
            Rc::strong_count(buf) > 1
        });
    }
}

pub struct DescriptorSetManager {
    device: Weak<RefCell<Device>>,
    pools: Vec<Vec<Rc<vk::DescriptorPool>>>,
}

impl DescriptorSetManager {
    fn new(device: &DeviceMutRef) -> DescriptorSetManager {
        let mut pools = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            pools.push(vec![DescriptorSetManager::create_descriptor_pool(&device.borrow())]);
        }

        DescriptorSetManager {
            device: Rc::downgrade(device),
            pools,
        }
    }

    pub fn reset_descriptor_pools(&mut self, device: &Device) {
        for pool in &self.pools[device.get_image_idx()] {
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
        layout: &ash::vk::DescriptorSetLayout,
    ) -> Result<vk::DescriptorSet,&'static str> {
        self.try_allocate_descriptor_set(layout, 0)
    }

    fn try_allocate_descriptor_set(
        &mut self,
        layout: &ash::vk::DescriptorSetLayout,
        next_idx: usize,
    ) -> Result<vk::DescriptorSet,&'static str> {
        if let Some(device) = self.device.upgrade() {
            let frame_pools = &mut self.pools[device.borrow().get_image_idx()];
            if next_idx >= frame_pools.len() {
                frame_pools.push(DescriptorSetManager::create_descriptor_pool(&device.borrow()));
                log::info!("Allocating additional descriptor pool {}.", next_idx);
            }

            let pool = &frame_pools[next_idx];
            let layouts = [*layout];
            let allocate_info = vk::DescriptorSetAllocateInfo {
                descriptor_pool: **pool, // TODO: remove Device::descriptor_pool
                descriptor_set_count: 1,
                p_set_layouts: layouts.as_ptr(),
                ..Default::default()
            };

            let descriptor_set = unsafe {
                device
                    .borrow()
                    .logical_device
                    .allocate_descriptor_sets(&allocate_info)
            };

            return match descriptor_set {
                Ok(set) => Ok(set[0]),
                Err(_) => self.try_allocate_descriptor_set(layout, next_idx + 1)
            };
        }

        Err("Failed to upgrade weak device for descriptor set allocation.")
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
