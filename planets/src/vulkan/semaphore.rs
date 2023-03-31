use alloc::rc::Rc;
use ash::vk;
use ash::vk::Handle;
use crate::vulkan::debug;
use crate::vulkan::debug::DebugResource;
use crate::vulkan::device::{DeviceMutRef};

pub struct Semaphore {
    device: DeviceMutRef,
    semaphore: vk::Semaphore,
    label: String,
}

impl Semaphore {
    pub fn new(device: &DeviceMutRef, label: &str) -> Self {
        let sem_create_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };

        let semaphore = unsafe {
            device
                .borrow()
                .logical_device
                .create_semaphore(&sem_create_info, None)
                .expect("Failed to create image available semaphore")
        };
        let ret = Self {
            device: Rc::clone(device),
            semaphore,
            label: String::from(label)
        };

        debug::Object::label(&device.borrow(), &ret);

        ret
    }

    pub fn get_semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl DebugResource for Semaphore {
    fn get_type(&self) -> vk::ObjectType {
        vk::ObjectType::SEMAPHORE
    }

    fn get_handle(&self) -> u64 {
        self.semaphore.as_raw()
    }

    fn get_label(&self) -> &String {
        &self.label
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.borrow().logical_device.destroy_semaphore(self.semaphore, None);
        }
    }
}