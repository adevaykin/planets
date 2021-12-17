use std::cell::RefCell;
use std::ops::Add;
use std::rc::Rc;
use std::time;
use crate::vulkan::device::Device;
use crate::vulkan::mem::StructBufferData;
use crate::vulkan::resources::ResourceManager;
use crate::vulkan::uniform_buffer::UniformBufferObject;

pub type GameLoopMutRef = Rc<RefCell<GameLoop>>;

pub struct GameLoop {
    // Application start time
    application_start_time: time::Instant,
    // Timestamp recorded when current frame has begun
    frame_start_time: time::Instant,
    // Time passed since previous frame was started
    prev_frame_duration: time::Duration,
    // Maximal number of frames per second.
    // Game loop will wait for required time to pass before starting a new frame to keep constant FPS if needed
    max_fps: i32,
    // Defines if frame started or not
    frame_started: bool,
    // Current frame number
    frame_num: u64,
    // Timer UBO
    timer_ubo: UniformBufferObject,
}

impl GameLoop {
    pub fn new(resource_manager: &mut ResourceManager) -> Self {
        let ubo_interface = TimerUBOInterface {
            total_time_elapsed: 0.0,
            frame_time_delta: 0.0,
        };

        let ubo_data = StructBufferData::new(&ubo_interface);

        GameLoop {
            application_start_time: time::Instant::now(),
            frame_start_time: time::Instant::now(),
            prev_frame_duration: time::Duration::from_millis(0),
            max_fps: 120,
            frame_started: false,
            frame_num: 0,
            timer_ubo: UniformBufferObject::new_with_data(resource_manager, &ubo_data, "Timer"),
        }
    }

    // Get if it's time to start frame already
    pub fn should_start_frame(&self) -> bool {
        let wanted_time_per_frame = time::Duration::from_micros(1000000 / self.max_fps as u64);

        wanted_time_per_frame <= self.frame_start_time.elapsed()
    }

    pub fn get_frame_started(&self) -> bool {
        self.frame_started
    }

    /// Update game loop state to notify next frame has started
    pub fn start_frame(&mut self) {
        self.prev_frame_duration = self.frame_start_time.elapsed();
        self.frame_start_time = time::Instant::now();
        self.frame_started = true;
    }

    // If frame was started - finish it and increase frame count
    pub fn finish_frame(&mut self) {
        if self.frame_started {
            self.frame_started = false;
            self.frame_num += 1;
        }
    }

    pub fn get_frame_num(&self) -> u64 {
        self.frame_num
    }

    // Get time Instant specifying the time we want to start next frame at
    pub fn get_wait_instant(&self) -> time::Instant {
        let wanted_time_per_frame = time::Duration::from_micros(1000000 / self.max_fps as u64);
        let mut wait_until = time::Instant::now();
        let time_since_frame_start = self.frame_start_time.elapsed();
        if wanted_time_per_frame > time_since_frame_start {
            let time_to_wait = wanted_time_per_frame - time_since_frame_start;
            wait_until = wait_until.add(time_to_wait);
        }

        wait_until
    }

    pub fn get_prev_frame_time(&self) -> time::Duration {
        self.prev_frame_duration
    }

    pub fn get_total_elapsed(&self) -> time::Duration {
        time::Instant::now() - self.application_start_time
    }

    pub fn set_max_fps(&mut self, max_fps: i32) {
        self.max_fps = max_fps
    }

    pub fn get_fps(&self) -> f32 {
        1000.0 / self.prev_frame_duration.as_millis() as f32
    }

    pub fn update_ubo(&mut self, device: &Device) {
        let ubo_interface = TimerUBOInterface {
            total_time_elapsed: self.get_total_elapsed().as_secs_f32(),
            frame_time_delta: self.get_prev_frame_time().as_secs_f32(),
        };

        let ubo_data = StructBufferData::new(&ubo_interface);
        self.timer_ubo.buffer.borrow().update_data(device, &ubo_data, 0);
    }

    pub fn get_timer_ubo(&self) -> &UniformBufferObject {
        &self.timer_ubo
    }
}

#[repr(C)]
struct TimerUBOInterface {
    total_time_elapsed: f32,
    frame_time_delta: f32,
}
