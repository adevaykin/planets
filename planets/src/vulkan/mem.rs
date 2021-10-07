use std::cell::RefCell;
use std::rc::Rc;

use ash::vk;

use super::cmd_buffers::SingleTimeCmdBuffer;
use super::device::Device;

pub trait BufferData {
    fn size(&self) -> usize;
    fn stride(&self) -> u32;
    fn as_ptr(&self) -> *const u8;
}

pub type AllocatedBufferMutRef = Rc<RefCell<AllocatedBuffer>>;

pub struct AllocatedBuffer {
    is_allocated: bool,
    pub buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    pub size: u64,
}

impl AllocatedBuffer {
    pub(super) fn new_with_size(
        device: &mut Device,
        size: u64,
        usage: vk::BufferUsageFlags,
        mem_props: vk::MemoryPropertyFlags,
    ) -> AllocatedBuffer {
        let (buffer, memory) = AllocatedBuffer::create_buffer(device, size, usage, mem_props);

        unsafe {
            device
                .logical_device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("Failed to bind buffer memory");
        }

        AllocatedBuffer {
            is_allocated: true,
            buffer,
            memory,
            size,
        }
    }

    pub(super) fn new_with_staging(
        device: &mut Device,
        data: &impl BufferData,
        usage: vk::BufferUsageFlags,
    ) -> AllocatedBuffer {
        let (staging_buffer, staging_memory) = AllocatedBuffer::create_buffer(
            device,
            data.size() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            device
                .logical_device
                .bind_buffer_memory(staging_buffer, staging_memory, 0)
                .expect("Failed to bind memory");
        }
        AllocatedBuffer::update_data_intern(device, staging_memory, data, 0);

        let (buffer, memory) = AllocatedBuffer::create_buffer(
            device,
            data.size() as u64,
            vk::BufferUsageFlags::TRANSFER_DST | usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        unsafe {
            device
                .logical_device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("Failed to bind buffer memory");
        }

        AllocatedBuffer::copy_buffer(device, staging_buffer, buffer, data.size() as u64);

        unsafe {
            device.logical_device.destroy_buffer(staging_buffer, None);
            device.logical_device.free_memory(staging_memory, None);
        }

        AllocatedBuffer {
            is_allocated: true,
            buffer,
            memory,
            size: data.size() as u64,
        }
    }

    pub(super) fn new_host_visible_coherent(
        device: &mut Device,
        data: &impl BufferData,
        usage: vk::BufferUsageFlags,
    ) -> AllocatedBuffer {
        let (buffer, memory) = AllocatedBuffer::create_buffer(
            device,
            data.size() as u64,
            usage,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            device
                .logical_device
                .bind_buffer_memory(buffer, memory, 0)
                .expect("Failed to bind buffer memory");
        }
        AllocatedBuffer::update_data_intern(device, memory, data, 0);

        AllocatedBuffer {
            is_allocated: true,
            buffer,
            memory,
            size: data.size() as u64,
        }
    }

    pub fn update_data(&self, device: &Device, data: &impl BufferData, offset: u64) {
        AllocatedBuffer::update_data_intern(device, self.memory, data, offset);
    }

    pub(super) fn destroy(&mut self, device: &Device) {
        self.is_allocated = false;
        unsafe {
            device.logical_device.destroy_buffer(self.buffer, None);
            device.logical_device.free_memory(self.memory, None);
        }
    }

    fn update_data_intern(
        device: &Device,
        memory: vk::DeviceMemory,
        data: &impl BufferData,
        offset: u64,
    ) {
        unsafe {
            let mapped_memory = device
                .logical_device
                .map_memory(
                    memory,
                    offset,
                    data.size() as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory") as *mut u8;
            mapped_memory.copy_from_nonoverlapping(data.as_ptr(), data.size());
            device.logical_device.unmap_memory(memory);
        }
    }

    fn create_buffer(
        device: &Device,
        size: u64,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            size: size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let buffer = unsafe {
            device
                .logical_device
                .create_buffer(&create_info, None)
                .expect("Failed to create buffer")
        };

        let mem_requirements =
            unsafe { device.logical_device.get_buffer_memory_requirements(buffer) };

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: mem_requirements.size,
            memory_type_index: device
                .find_memory_type(mem_requirements.memory_type_bits, properties),
            ..Default::default()
        };

        let memory = unsafe {
            device
                .logical_device
                .allocate_memory(&allocate_info, None)
                .expect("Failed to allocate memory")
        };

        (buffer, memory)
    }

    fn copy_buffer(device: &Device, src: vk::Buffer, dst: vk::Buffer, size: vk::DeviceSize) {
        let single_time_cmd_buffer = SingleTimeCmdBuffer::begin(device);

        let copy_regions = [vk::BufferCopy {
            size,
            ..Default::default()
        }];

        unsafe {
            device.logical_device.cmd_copy_buffer(
                single_time_cmd_buffer.get_cmd_buffer(),
                src,
                dst,
                &copy_regions,
            );
        }
    }
}

impl Drop for AllocatedBuffer {
    fn drop(&mut self) {
        if self.is_allocated {
            log::error!("Dropping buffer that was never destroyed!");
        }
    }
}

pub struct VecBufferData<'a, T> {
    data: &'a Vec<T>,
}

impl<'a, T> VecBufferData<'a, T> {
    pub fn new(data: &Vec<T>) -> VecBufferData<T> {
        VecBufferData { data }
    }
}

impl<'a, T> BufferData for VecBufferData<'a, T> {
    fn size(&self) -> usize {
        self.data.len() * std::mem::size_of::<T>()
    }

    fn stride(&self) -> u32 {
        std::mem::size_of::<T>() as u32
    }

    fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr() as *const u8
    }
}

pub struct StructBufferData<'a, T> {
    data: &'a T,
}

impl<'a, T> StructBufferData<'a, T> {
    pub fn new(data: &T) -> StructBufferData<T> {
        StructBufferData { data }
    }
}

impl<'a, T> BufferData for StructBufferData<'a, T> {
    fn size(&self) -> usize {
        std::mem::size_of::<T>()
    }

    fn stride(&self) -> u32 {
        std::mem::size_of::<T>() as u32
    }

    fn as_ptr(&self) -> *const u8 {
        self.data as *const T as *const u8
    }
}

#[cfg(test)]
mod tests {
    use super::BufferData;
    use super::StructBufferData;
    use super::VecBufferData;

    #[test]
    fn empty_vec_buffer_data() {
        let empty_vec: Vec<u8> = vec![];
        let test_vec_buffer_data = VecBufferData::new(&empty_vec);
        assert_eq!(test_vec_buffer_data.size(), 0);
        assert_eq!(
            test_vec_buffer_data.stride(),
            std::mem::size_of::<u8>() as u32
        );
        assert_eq!(test_vec_buffer_data.as_ptr(), empty_vec.as_ptr());
    }

    #[test]
    fn u8_vec_buffer_data() {
        let test_vec: Vec<u8> = vec![1, 2, 3];
        let test_vec_buffer_data = VecBufferData::new(&test_vec);
        assert_eq!(test_vec_buffer_data.size(), 3);
        assert_eq!(test_vec_buffer_data.stride(), 1);
        assert_eq!(test_vec_buffer_data.as_ptr(), test_vec.as_ptr());
    }

    #[test]
    fn f32_vec_buffer_data() {
        let test_vec: Vec<f32> = vec![1.0, 2.0, 3.0];
        let test_vec_buffer_data = VecBufferData::new(&test_vec);
        assert_eq!(test_vec_buffer_data.size(), 3 * 4);
        assert_eq!(test_vec_buffer_data.stride(), 4);
        assert_eq!(
            test_vec_buffer_data.as_ptr(),
            test_vec.as_ptr() as *const u8
        );
    }

    #[allow(dead_code)]
    struct TestData {
        value: i32,
        value2: f32,
    }

    #[allow(dead_code)]
    struct TestVectorData {
        value: Vec<TestData>,
    }

    #[test]
    fn struct_buffer_data() {
        let test_data = TestData {
            value: 42,
            value2: 42.0,
        };
        let test_data_buffer = StructBufferData::new(&test_data);
        assert_eq!(
            test_data_buffer.size(),
            std::mem::size_of::<i32>() + std::mem::size_of::<f32>()
        );
        assert_eq!(test_data_buffer.stride(), test_data_buffer.size() as u32);
    }

    #[test]
    fn struct_buffer_data_vector() {
        let test_data = vec![
            TestData {
                value: 42,
                value2: 42.0,
            },
            TestData {
                value: 42,
                value2: 42.0,
            },
            TestData {
                value: 42,
                value2: 42.0,
            },
        ];

        let vector_data = TestVectorData { value: test_data };
        let test_data_buffer = StructBufferData::new(&vector_data);
        assert_eq!(test_data_buffer.size(), std::mem::size_of::<TestData>() * 3);
        assert_eq!(test_data_buffer.stride(), test_data_buffer.size() as u32);
    }
}