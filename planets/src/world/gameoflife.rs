use crate::app::GAME_FIELD_SIZE;
use crate::vulkan::device::Device;
use crate::vulkan::mem::{AllocatedBufferMutRef, StructBufferData};
use crate::vulkan::resources::ResourceManager;
use ash::vk;
use std::time::Duration;

const MS_PER_STEP: u64 = 500;

#[repr(C)]
#[derive(Clone)]
struct Field {
    state: [[i32; GAME_FIELD_SIZE]; GAME_FIELD_SIZE],
}

impl Field {
    fn new() -> Self {
        Field {
            state: [[0; GAME_FIELD_SIZE]; GAME_FIELD_SIZE],
            
            
        }
         
    }

}

pub struct GameOfLife {
    time_since_last_step: Duration,
    field: Field,
    gpu_buffer: AllocatedBufferMutRef,
}

impl GameOfLife {
    pub fn new(resource_manager: &mut ResourceManager) -> Self {
        let mut field = Field::new();        
        
        field.state[2][0] = 1;
        field.state[2][1] = 1;
        field.state[2][2] = 1;
        field.state[1][2] = 1;
        field.state[0][1] = 1;
        
        let buffer_data = StructBufferData::new(&field);
        let gpu_buffer = resource_manager.buffer_host_visible_coherent(
            &buffer_data,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            "GameState",
        );

        GameOfLife {
            time_since_last_step: Duration::from_secs(0),
            field,
            gpu_buffer,
        }
    }

    pub fn do_step(&mut self, time_passed: Duration) {
        self.time_since_last_step += time_passed;
        if self.time_since_last_step.as_millis() < MS_PER_STEP as u128 {
            return;
        }

        self.time_since_last_step -= Duration::from_millis(MS_PER_STEP);
        // Update game state here
        
        let old_field = self.field.clone();

        // Example code for setting one cell after another to "true" (1)

        fn who_are_neighbours(row: i8, col: i8) -> [[i8; 2]; 8] {
            
            let mut arr_neighbours = [[0; 2]; 8];            
    
            arr_neighbours[0] = [row - 1, col - 1];
            arr_neighbours[1] = [row - 1, col];
            arr_neighbours[2] = [row - 1, col + 1];
            arr_neighbours[3] = [row, col + 1];
            arr_neighbours[4] = [row + 1, col + 1];
            arr_neighbours[5] = [row + 1, col];
            arr_neighbours[6] = [row + 1, col - 1];
            arr_neighbours[7] = [row, col - 1];
    
            for n in 0..8 {
                if arr_neighbours[n][0] == -1 {
                    arr_neighbours[n][0] = 9;
                }
                if arr_neighbours[n][0] == 10 {
                    arr_neighbours[n][0] = 0;
                }
    
                if arr_neighbours[n][1] == -1 {
                    arr_neighbours[n][1] = 9;
                }
    
                if arr_neighbours[n][1] == 10 {
                    arr_neighbours[n][1] = 0;
                }
            }
    
            //println!("arr neighbour{:?}", arr_neighbours);
            arr_neighbours
            
        }
        

        for i in 0..GAME_FIELD_SIZE {
            for j in 0..GAME_FIELD_SIZE {
                
                let mut live_or_dead: bool = false;

                if old_field.state[i][j] == 1 {
                    live_or_dead = true;
                } else {
                    live_or_dead = false;
                }

                let mut arr_neighbours_counter: i32 = 0;

                
                for n in who_are_neighbours(i as i8, j as i8) {
                    arr_neighbours_counter += old_field.state[n[0] as usize][n[1] as usize];
                    
                }

                if arr_neighbours_counter == 3 && live_or_dead == false {
                    self.field.state[i][j] = 1;
                    
                    
                    
                    
                }
                if arr_neighbours_counter >= 2 && arr_neighbours_counter < 4 && live_or_dead == true
                {
                    self.field.state[i][j] = 1;
                    
                    
                    
                    
                }
                if (arr_neighbours_counter < 2 || arr_neighbours_counter > 3) && live_or_dead == true
                {
                    self.field.state[i][j] = 0;
                    
                    
                    
                }
                
                
                

                



                /* if self.field.state[i][j] == 1 {
                    self.field.state[i][j] = 0;
                    return;
                } */
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
