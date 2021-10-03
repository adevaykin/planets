use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::gameloop::GameLoop;
use crate::vulkan::device::{Device, MAX_FRAMES_IN_FLIGHT};
use crate::vulkan::mem::StructBufferData;
use crate::vulkan::resources::ResourceManager;
use crate::vulkan::uniform_buffer::UniformBufferObject;

pub type TimerMutRef = Rc<RefCell<Timer>>;

#[repr(C)]
struct TimerUBOInterface {
    total_time_elapsed: f32,
    frame_time_delta: f32,
}

pub struct Timer {
    pub ubos: [UniformBufferObject; MAX_FRAMES_IN_FLIGHT],
}

impl Timer {
    pub fn new(resource_manager: &mut ResourceManager) -> Timer {
        let ubo_interface = TimerUBOInterface {
            total_time_elapsed: 0.0,
            frame_time_delta: 0.0,
        };

        let ubo_data = StructBufferData::new(&ubo_interface);
        let ubos = [
            UniformBufferObject::new_with_data(resource_manager, &ubo_data, "Timer"),
            UniformBufferObject::new_with_data(resource_manager, &ubo_data, "Timer"),
        ];

        Timer { ubos }
    }

    pub fn update(&mut self, gameloop: &GameLoop, device: &Device, frame_num: usize) {
        let ubo_interface = TimerUBOInterface {
            total_time_elapsed: gameloop.get_total_elapsed().as_secs_f32(),
            frame_time_delta: gameloop.get_prev_frame_time().as_secs_f32(),
        };

        let ubo_data = StructBufferData::new(&ubo_interface);
        self.ubos[frame_num]
            .buffer
            .borrow()
            .update_data(device, &ubo_data, 0);
    }
}
