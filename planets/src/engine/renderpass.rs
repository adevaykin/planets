use ash::vk;
use crate::vulkan::img::image::{ImageMutRef};
use crate::vulkan::pipeline::Pipeline;

pub trait RenderPass {
    fn run(&mut self, cmd_buffer: vk::CommandBuffer, input_attachments: Vec<ImageMutRef>) -> Vec<ImageMutRef>;
    fn get_pipeline(&self) -> &Pipeline;
    fn get_descriptor_set(&self) -> vk::DescriptorSet;
}
