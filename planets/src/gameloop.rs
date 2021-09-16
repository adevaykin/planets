use std::time;
use std::thread;

pub struct GameLoop {
    /// Timestamp recorded when game starts
    start_time: time::Instant,
    /// Timestamp recorded when current frame has begun
    frame_start_time: time::Instant,
    /// Time passed since previous frame was started
    prev_frame_duration: time::Duration,
    /// Maximal number of frames per second.
    /// Game loop will wait for required time to pass before starting a new frame to keep constant FPS if needed
    max_fps: i32,
}

impl GameLoop {
    pub fn new() -> Self {
        GameLoop {
            start_time: time::Instant::now(),
            frame_start_time: time::Instant::now(),
            prev_frame_duration: time::Duration::from_millis(0),
            max_fps: 120,
        }
    }

    /// Update game loop state to notify next frame has started
    pub fn start_frame(&mut self) {
        let wanted_time_per_frame = time::Duration::from_micros(1000000 / self.max_fps as u64);
        if wanted_time_per_frame > self.frame_start_time.elapsed() {
            thread::sleep(wanted_time_per_frame - self.frame_start_time.elapsed());
        }

        self.prev_frame_duration = self.frame_start_time.elapsed();
        self.frame_start_time = time::Instant::now();
    }

    pub fn get_prev_frame_time(&self) -> time::Duration {
        self.prev_frame_duration
    }

    pub fn set_max_fps(&mut self, max_fps: i32) {
        self.max_fps = max_fps
    }
}