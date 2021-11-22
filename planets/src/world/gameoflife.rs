use crate::app::GAME_FIELD_SIZE;
use crate::vulkan::device::Device;
use crate::vulkan::mem::{AllocatedBufferMutRef, StructBufferData};
use crate::vulkan::resources::ResourceManager;
use ash::vk;

#[repr(C)]
#[derive(Clone)]
struct Field {
    state: [[u32; GAME_FIELD_SIZE]; GAME_FIELD_SIZE],
}

impl Field {
    fn new() -> Self {
        Field {
            state: [[0; GAME_FIELD_SIZE]; GAME_FIELD_SIZE],
        }
    }
}

pub struct GameOfLife {
    field: Field,
    gpu_buffer: AllocatedBufferMutRef,
}

impl GameOfLife {
    pub fn new(resource_manager: &mut ResourceManager) -> Self {
        let field = Field::new();
        let buffer_data = StructBufferData::new(&field);
        let gpu_buffer = resource_manager.buffer_host_visible_coherent(
            &buffer_data,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            "GameState",
        );

        GameOfLife { field, gpu_buffer }
    }

    pub fn do_step(&mut self) {
        // Update game state here
        //let old_field = self.field.clone();

        // Example code for setting one cell after another to "true" (1)
        for i in 0..GAME_FIELD_SIZE {
            for j in 0..GAME_FIELD_SIZE {
                if self.field.state[i][j] == 0 {
                    self.field.state[i][j] = 1;
                    return;
                }
            }
        }
    }

    pub fn update(&self, device: &Device) {
        let buffer_data = StructBufferData::new(&self.field);
        self.gpu_buffer
            .borrow()
            .update_data(device, &buffer_data, 0);
    }

    pub fn get_gpu_buffer(&self) -> &AllocatedBufferMutRef {
        &self.gpu_buffer
    }
}
