use ash::vk;

use super::swapchain::Swapchain;

pub trait RenderPass {
    // TODO: leave only frame_num and command_buffer arguments
    fn draw_frame(
        &mut self,
        swapchain: &Swapchain,
        frame_num: usize,
        command_buffer: &vk::CommandBuffer,
    );
}
