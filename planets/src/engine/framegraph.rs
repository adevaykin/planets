use ash::vk;
use crate::vulkan::image::{ImageMutRef};
use crate::vulkan::resources::ResourceManagerMutRef;
use crate::engine::viewport::Viewport;

pub enum AttachmentDirection {
    Read,
    Write,
    ReadWrite,
}

pub enum AttachmentSize {
    Absolute(u32,u32),
    Relative(f32,f32),
}

pub trait RenderPass {
    fn get_name(&self) -> &str;
    fn run(&mut self, cmd_buffer: vk::CommandBuffer, attachments: Vec<vk::ImageView>);
    fn get_attachments(&self) -> &Vec<(&'static str, vk::AttachmentDescription)>;
}

pub struct FrameGraph {
    passes: Vec<Box<dyn RenderPass>>,
    attachments: Vec<ImageMutRef>,
}

impl FrameGraph {
    pub fn new(resource_manager: &ResourceManagerMutRef, viewport: &Viewport) -> Self {

        FrameGraph {
            passes: vec![],
            attachments: vec![
                resource_manager.borrow_mut().image(
                    viewport.width,
                    viewport.height,
                    vk::Format::R8G8B8A8_SRGB,
                    vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
                    "FirstAttachment")
            ],
        }
    }

    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }

    pub fn build(&mut self) {

    }

    pub fn execute(&mut self, cmd_buffer: vk::CommandBuffer) {
        for pass in &mut self.passes {
            let mut attachment_views = vec![];
            for a in &self.attachments {
                a.borrow_mut().transition_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, cmd_buffer);
                attachment_views.push(a.borrow_mut().add_get_view(vk::Format::R8G8B8A8_SRGB));
            }
            pass.run(cmd_buffer, attachment_views);
            for a in &mut self.attachments {
                a.borrow_mut().set_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
            }
        }
    }

    pub fn get_result(&self) -> &ImageMutRef {
        &self.attachments[0]
    }
}
