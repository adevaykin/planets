use ash::vk::BufferUsageFlags;
use cgmath::{Matrix4, SquareMatrix};
use crate::vulkan::device::Device;
use crate::vulkan::mem::{AllocatedBufferMutRef, VecBufferData};
use crate::vulkan::resources::{ResourceManagerMutRef};

#[repr(C)]
#[derive(Clone,Copy)]
pub struct ModelDataSSBOInterface {
    pub transform: Matrix4<f32>,
}

pub struct ModelData {
    data: Vec<ModelDataSSBOInterface>,
    ssbo: AllocatedBufferMutRef,
}

impl ModelData {
    pub fn new(resource_manager: &ResourceManagerMutRef) -> Self {
        let data = vec![ModelDataSSBOInterface {
            transform: Matrix4::identity(),
        }];

        let ssbo_data = VecBufferData::new(&data);
        let ssbo = resource_manager.borrow_mut()
            .buffer_host_visible_coherent(&ssbo_data, BufferUsageFlags::STORAGE_BUFFER, "ModelTransforms");

        ModelData {
            data,
            ssbo
        }
    }

    pub fn update(&self, device: &Device) {
        let ssbo_data = VecBufferData::new(&self.data);
        self.ssbo.borrow().update_data(device, &ssbo_data, 0);
    }

    pub fn set_data_for(&mut self, index: usize, data: &ModelDataSSBOInterface) {
        self.data[index] = *data;
    }

    pub fn get_ssbo(&self) -> &AllocatedBufferMutRef {
        &self.ssbo
    }
}