use ash::vk;
use crate::vulkan::image::image::{ImageMutRef};
use crate::vulkan::pipeline::Pipeline;

pub trait RenderPass {
    fn get_name(&self) -> &str;
    fn run(&mut self, cmd_buffer: vk::CommandBuffer) -> Vec<ImageMutRef>;
    fn get_pipeline(&self) -> &Pipeline;
    fn get_descriptor_set(&self) -> vk::DescriptorSet;
}
