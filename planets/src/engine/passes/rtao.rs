use ash::vk;
use crate::engine::renderpass::RenderPass;
use crate::vulkan::img::image::ImageMutRef;
use crate::vulkan::pipeline::Pipeline;

struct RaytracedAo {

}

impl RenderPass for RaytracedAo {
    fn run(&mut self, cmd_buffer: vk::CommandBuffer, input_attachments: Vec<ImageMutRef>) -> Vec<ImageMutRef> {
        todo!()
    }

    fn get_pipeline(&self) -> &Pipeline {
        todo!()
    }

    fn get_descriptor_set(&self) -> Result<vk::DescriptorSet, &'static str> {
        todo!()
    }
}
