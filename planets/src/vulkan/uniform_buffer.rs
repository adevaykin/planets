use ash::vk;

use super::mem::{AllocatedBufferMutRef, BufferData};
use super::resources::ResourceManager;

pub struct UniformBufferObject {
    pub buffer: AllocatedBufferMutRef,
}

impl UniformBufferObject {
    pub fn new_with_data(
        resource_manager: &mut ResourceManager,
        data: &impl BufferData,
        label: &str,
    ) -> UniformBufferObject {
        let buffer = ResourceManager::buffer_host_visible_coherent(
            resource_manager,
            data,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            label,
        );

        UniformBufferObject { buffer }
    }
}
