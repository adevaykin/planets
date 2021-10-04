use std::cell::RefCell;
use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use super::debug;
use super::device::{Device, DeviceMutRef, MAX_FRAMES_IN_FLIGHT};
use super::mem::{AllocatedBuffer, AllocatedBufferMutRef, BufferData};

pub struct ResourceManager {
    device: DeviceMutRef,
    buffers: Vec<AllocatedBufferMutRef>,
    pub descriptor_set_manager: DescriptorSetManager,
}

pub type ResourceManagerMutRef = Rc<RefCell<ResourceManager>>;

impl ResourceManager {
    pub fn new(device: &DeviceMutRef) -> ResourceManager {
        let descriptor_set_manager = DescriptorSetManager::new(&device.borrow());
        ResourceManager {
            device: Rc::clone(device),
            buffers: vec![],
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

    pub fn remove_unused(&mut self) {
        let defive_ref = self.device.borrow();
        self.buffers.retain(|buf| {
            if Rc::strong_count(&buf) <= 1 {
                buf.borrow_mut().destroy(&defive_ref);
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
    pools: [Vec<Rc<vk::DescriptorPool>>; MAX_FRAMES_IN_FLIGHT],
}

impl DescriptorSetManager {
    fn new(device: &Device) -> DescriptorSetManager {
        let pools = [
            vec![DescriptorSetManager::create_descriptor_pool(device)],
            vec![DescriptorSetManager::create_descriptor_pool(device)],
        ];

        DescriptorSetManager { pools }
    }

    pub fn reset_descriptor_pools(&self, device: &Device, frame_num: usize) {
        for pool in &self.pools[frame_num] {
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
        frame_num: usize,
    ) -> vk::DescriptorSet {
        self.try_allocate_descriptor_set(device, layout, frame_num, 0)
    }

    fn try_allocate_descriptor_set(
        &mut self,
        device: &Device,
        layout: &ash::vk::DescriptorSetLayout,
        frame_num: usize,
        next_index: usize,
    ) -> vk::DescriptorSet {
        let frame_pools = &mut self.pools[frame_num];
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
            return self.try_allocate_descriptor_set(device, layout, frame_num, next_index + 1);
        }

        return descriptor_set.unwrap()[0];
    }

    fn create_descriptor_pool(device: &Device) -> Rc<vk::DescriptorPool> {
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

        let descriptor_pool = Rc::new(unsafe {
            device
                .logical_device
                .create_descriptor_pool(&create_info, None)
                .expect("Failed to create descriptor set")
        });

        descriptor_pool
    }

    fn destroy(&mut self, device: &Device) {
        for frame in 0..MAX_FRAMES_IN_FLIGHT {
            self.reset_descriptor_pools(device, frame);
            for pool in &self.pools[frame] {
                unsafe {
                    device.logical_device.destroy_descriptor_pool(**pool, None);
                }
            }
        }
    }
}
