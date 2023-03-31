use crate::vulkan::device::DeviceMutRef;
use ash::vk;
use std::rc::Rc;

pub struct Renderer {
    device: DeviceMutRef,
}

impl Renderer {
    pub fn new(
        device: &DeviceMutRef,
    ) -> Self {
        Renderer {
            device: Rc::clone(device),
        }
    }

    pub fn begin_frame(&self) {
        let cmd_buffer = self.device.borrow().get_command_buffer();
        let logical_device = &self.device.borrow().logical_device;

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            ..Default::default()
        };
        unsafe {
            logical_device
                .reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::default())
                .expect("Failed to reset command buffer");
        }
        unsafe {
            logical_device
                .begin_command_buffer(cmd_buffer, &begin_info)
                .expect("Failed to begin command buffer");
        }
    }

    pub fn end_frame(&self) {
        let cmd_buffer = self.device.borrow().get_command_buffer();
        let logical_device = &self.device.borrow().logical_device;

        unsafe {
            logical_device.end_command_buffer(cmd_buffer).expect("Failed to end command buffer");
        }
    }
}
