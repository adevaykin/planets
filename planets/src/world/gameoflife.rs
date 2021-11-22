use crate::app::GAME_FIELD_SIZE;
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
        let mut state = [[0; GAME_FIELD_SIZE]; GAME_FIELD_SIZE];
        state[0][0] = 1;
        state[0][15] = 1;
        state[15][0] = 1;
        state[15][15] = 1;
        Field { state }
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
    }

    pub fn get_gpu_buffer(&self) -> &AllocatedBufferMutRef {
        &self.gpu_buffer
    }
}
