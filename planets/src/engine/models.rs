use ash::vk::BufferUsageFlags;
use cgmath::{Matrix4, SquareMatrix};
use crate::vulkan::mem::{AllocatedBufferMutRef, VecBufferData};
use crate::vulkan::resources::{ResourceManagerMutRef};

#[repr(C)]
struct ModelDataUBOInterface {
    pub transform: Matrix4<f32>,
}

pub struct ModelData {
    resource_manager: ResourceManagerMutRef,
    data: Vec<ModelDataUBOInterface>,
    ssbo: AllocatedBufferMutRef,
}

impl ModelData {
    pub fn new(resource_manager: &ResourceManagerMutRef) -> Self {
        let data = vec![ModelDataUBOInterface {
            transform: Matrix4::identity(),
        }];

        let ssbo_data = VecBufferData::new(&data);
        let ssbo = resource_manager.borrow_mut()
            .buffer_with_staging(&ssbo_data, BufferUsageFlags::STORAGE_BUFFER, "ModelData");

        ModelData {
            resource_manager: resource_manager.clone(),
            data,
            ssbo
        }
    }

    pub fn get_ssbo(&self) -> &AllocatedBufferMutRef {
        &self.ssbo
    }
}