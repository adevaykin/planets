use ash::vk::BufferUsageFlags;

use crate::vulkan::mem::{VecBufferData, AllocatedBufferMutRef, BufferData};
use crate::vulkan::resources::ResourceManager;
use crate::vulkan::device::Device;

pub struct ArraySSBO<T: Default + Copy + BufferData> {
    cpu_buffer: Vec<T>,
    pub gpu_buffer: AllocatedBufferMutRef,
}

impl<T: Default + Copy + BufferData> ArraySSBO<T> {
    pub fn new(resource_manager: &mut ResourceManager, label: &str) -> ArraySSBO<T> {
        let default_entry: T = Default::default();
        let mut initial_data = vec![];
        initial_data.resize(1024, default_entry);
        let buffer_data = VecBufferData::new(&initial_data);

        let final_label = format!("ArraySSBO: {}", label);
        let gpu_buffer = resource_manager.buffer_host_visible_coherent(&buffer_data, BufferUsageFlags::STORAGE_BUFFER, final_label.as_str());

        ArraySSBO { cpu_buffer: initial_data, gpu_buffer }
    }

    pub fn update_at(&mut self, device: &Device, index: u64, data: &T) {
        self.cpu_buffer[index as usize] = *data;
        self.gpu_buffer.borrow_mut().update_data(device, data, index * data.size() as u64);
    }
}