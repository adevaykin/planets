use ash::vk::BufferUsageFlags;
use cgmath::{Matrix4, SquareMatrix};
use crate::vulkan::device::{Device, MAX_FRAMES_IN_FLIGHT};
use crate::vulkan::mem::{AllocatedBufferMutRef, VecBufferData};
use crate::vulkan::resources::manager::{ResourceManagerMutRef};

#[repr(C)]
#[derive(Clone,Copy)]
pub struct ModelDataSSBOInterface {
    pub transform: Matrix4<f32>,
}

pub struct ModelData {
    data: Vec<ModelDataSSBOInterface>,
    ssbo: Vec<AllocatedBufferMutRef>,
}

impl ModelData {
    pub fn new(resource_manager: &ResourceManagerMutRef) -> Self {
        let data = vec![ModelDataSSBOInterface {
            transform: Matrix4::identity(),
        }; 1024];

        let ssbo_data = VecBufferData::new(&data);
        let mut resource_manager_ref = resource_manager.borrow_mut();
        let mut ssbo = vec![];
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            ssbo.push(resource_manager_ref
                .buffer_host_visible_coherent(&ssbo_data, BufferUsageFlags::STORAGE_BUFFER, format!("ModelTransforms{}", i).as_str()))
        }

        ModelData {
            data,
            ssbo
        }
    }

    pub fn update(&self, device: &Device) {
        let ssbo_data = VecBufferData::new(&self.data);
        self.ssbo[device.get_image_idx()].borrow().update_data(device, &ssbo_data, 0);
    }

    pub fn set_data_for(&mut self, index: usize, data: &ModelDataSSBOInterface) {
        self.data[index] = *data;
    }

    pub fn get_ssbo(&self, image_idx: usize) -> &AllocatedBufferMutRef {
        &self.ssbo[image_idx]
    }
}
