use crate::engine::resourcebinding::Content::UniformBuffer;
use crate::vulkan::mem::AllocatedBufferMutRef;
use crate::vulkan::uniform_buffer::UniformBufferObject;
use std::rc::Rc;

pub enum PipelineStage {
    Vertex,
    Fragment,
}

enum Content {
    StorageBuffer(AllocatedBufferMutRef),
    UniformBuffer(UniformBufferObject),
}

pub struct ResourceBinding {
    binding: u32,
    stage: PipelineStage,
    content: Content,
}

impl ResourceBinding {
    pub fn new_ubo(binding: u32, stage: PipelineStage, ubo: &UniformBufferObject) -> Self {
        ResourceBinding {
            binding,
            stage,
            content: Content::UniformBuffer(ubo.clone()),
        }
    }

    pub fn new_storage(binding: u32, stage: PipelineStage, buffer: &AllocatedBufferMutRef) -> Self {
        ResourceBinding {
            binding,
            stage,
            content: Content::StorageBuffer(Rc::clone(buffer)),
        }
    }
}
