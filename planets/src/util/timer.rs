use std::time::{Duration, Instant};
use std::rc::Rc;
use std::cell::RefCell;

use crate::vulkan::device::Device;
use crate::vulkan::uniform_buffer::UniformBufferObject;
use crate::vulkan::mem::StructBufferData;
use crate::vulkan::resources::ResourceManager;

pub type TimerMutRef = Rc<RefCell<Timer>>;

#[repr(C)]
struct TimerUBOInterface {
    total_time_elapsed: f32,
    frame_time_delta: f32,
}

pub struct Timer {
    pub ubos: Vec<UniformBufferObject>,
    start: Instant,
    frame_start: Instant,
    pub frame_elapsed: Duration,
}

impl Timer {
    pub fn new(resource_manager: &mut ResourceManager) -> Timer {
        let ubo_interface = TimerUBOInterface {
            total_time_elapsed: 0.0,
            frame_time_delta: 0.0,
        };

        let ubo_data = StructBufferData::new(&ubo_interface);
        let ubos = vec![
            UniformBufferObject::new_with_data(resource_manager, &ubo_data, "Timer"),
            UniformBufferObject::new_with_data(resource_manager, &ubo_data, "Timer"),
        ];

        Timer {
            ubos,
            start: Instant::now(),
            frame_start: Instant::now(),
            frame_elapsed: Duration::from_millis(0)
        }
    }

    pub fn start_frame(&mut self, device: &Device, frame_num: usize) {
        let now = Instant::now();
        self.frame_elapsed = now - self.frame_start;
        self.frame_start = now; 
        self.update(device, frame_num);
    }

    /// Returns duration till the end of the frame to keep constant FPS
    pub fn get_wait(&self) -> Duration {
        let elapsed = Instant::now() - self.frame_start;
        let fps = elapsed.as_millis() as f32 / 1000 as f32;
        let fps_overhead = 60 as f32 - fps;
        if fps_overhead > 0.0 {
            return Duration::from_millis((fps_overhead * (60.0 / 1000.0)) as u64);
        } else {
            return Duration::from_millis(0);
        }
    }

    pub fn get_fps(&self) -> u32 {
        (1000.0 / self.frame_elapsed.as_millis() as f32) as u32
    }

    pub fn get_total_elapsed(&self) -> Duration {
        self.frame_start - self.start
    }

    pub fn get_frame_start(&self) -> &Instant {
        &self.frame_start
    }

    fn update(&mut self, device: &Device, frame_num: usize) {
        let ubo_interface = TimerUBOInterface {
            total_time_elapsed: self.get_total_elapsed().as_secs_f32(),
            frame_time_delta: self.frame_elapsed.as_secs_f32(),
        };

        let ubo_data = StructBufferData::new(&ubo_interface);
        self.ubos[frame_num].buffer.borrow().update_data(device, &ubo_data, 0);
    }
}