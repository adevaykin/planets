use ash::vk;

use super::device::Device;

pub struct SingleTimeCmdBuffer<'a> {
    device: &'a Device,
    pub cmd_buffers: Vec<vk::CommandBuffer>
}

impl<'a> SingleTimeCmdBuffer<'a> {
    pub fn begin(device: &'a Device) -> SingleTimeCmdBuffer<'a> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            level: vk::CommandBufferLevel::PRIMARY,
            command_pool: *device.command_pool,
            command_buffer_count: 1,
            ..Default::default()
        };

        let command_buffers = unsafe { device.logical_device.allocate_command_buffers(&allocate_info).expect("Failed to allocate command buffer") };
        let command_buffer = command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };

        unsafe { device.logical_device.begin_command_buffer(command_buffer, &begin_info).expect("Failed to begin command buffer"); }

        SingleTimeCmdBuffer { device, cmd_buffers: command_buffers }
    }

    pub fn get_cmd_buffer(&self) -> vk::CommandBuffer {
        self.cmd_buffers[0]
    }
}

impl<'a> Drop for SingleTimeCmdBuffer<'a> {
    fn drop(&mut self) {
        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            command_buffer_count: 1,
            p_command_buffers: self.cmd_buffers.as_ptr(),
            ..Default::default()
        }];

        unsafe {
            self.device.logical_device.end_command_buffer(self.cmd_buffers[0]).expect("Failed to end command buffer");
            self.device.logical_device.queue_submit(self.device.graphics_queue, &submit_infos, vk::Fence::null()).expect("Failed to submit queue");
            self.device.logical_device.queue_wait_idle(self.device.graphics_queue).expect("Failed to wait queue idle");
            self.device.logical_device.free_command_buffers(*self.device.command_pool, &self.cmd_buffers);
        }
    }
}