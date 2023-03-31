use alloc::rc::Rc;
use ash::vk;
use ash::vk::Handle;
use crate::vulkan::debug;

use crate::vulkan::debug::DebugResource;
use crate::vulkan::device::{DeviceMutRef};

pub struct Fence {
    device: DeviceMutRef,
    fence: vk::Fence,
    label: String,
}

impl Fence {
    pub fn new(device: &DeviceMutRef, flags: vk::FenceCreateFlags, label: &str) -> Self {
        let fence_create_info = vk::FenceCreateInfo {
            flags,
            ..Default::default()
        };

        let fence = unsafe {
            device
                .borrow()
                .logical_device
                .create_fence(&fence_create_info, None)
                .expect("Failed to create in-flight fence")
        };

        let ret = Self {
            device: Rc::clone(device),
            fence,
            label: String::from(label)
        };

        debug::Object::label(&device.borrow(), &ret);

        ret
    }

    pub fn get_fence(&self) -> vk::Fence {
        self.fence
    }
}

impl DebugResource for Fence {
    fn get_type(&self) -> vk::ObjectType {
        vk::ObjectType::FENCE
    }

    fn get_handle(&self) -> u64 {
        self.fence.as_raw()
    }

    fn get_label(&self) -> &String {
        &self.label
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.borrow().logical_device.destroy_fence(self.fence, None);
        }
    }
}
