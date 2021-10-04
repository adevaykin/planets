use std::ops::Add;
use std::thread;
use std::time;

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
}

impl GameLoop {
    pub fn new() -> Self {
        GameLoop {
            application_start_time: time::Instant::now(),
            frame_start_time: time::Instant::now(),
            prev_frame_duration: time::Duration::from_millis(0),
            max_fps: 120,
            frame_started: false,
            frame_num: 0,
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
        if wanted_time_per_frame > self.frame_start_time.elapsed() {
            let time_to_wait = wanted_time_per_frame - self.frame_start_time.elapsed();
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
}

mod tests {
    use super::GameLoop;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn should_start_frame() {
        let mut gameloop = GameLoop::new();
        gameloop.set_max_fps(1);
        gameloop.start_frame();
        thread::sleep(Duration::from_millis(100));
        // Too soon to start frame after 100ms - GameLoop should wait 1000ms for 1 FPS
        assert_eq!(gameloop.should_start_frame(), false);
        thread::sleep(Duration::from_millis(900));
        assert_eq!(gameloop.should_start_frame(), true);
    }

    #[test]
    fn frame_started() {
        let mut gameloop = GameLoop::new();
        assert_eq!(gameloop.get_frame_started(), false);
        gameloop.start_frame();
        assert_eq!(gameloop.get_frame_started(), true);
        gameloop.finish_frame();
        assert_eq!(gameloop.get_frame_started(), false);
    }

    #[test]
    fn finish_frame() {
        let mut gameloop = GameLoop::new();
        assert_eq!(gameloop.get_frame_num(), 0);
        gameloop.start_frame();
        assert_eq!(gameloop.get_frame_num(), 0);
        gameloop.finish_frame();
        assert_eq!(gameloop.get_frame_num(), 1);
        gameloop.finish_frame();
        assert_eq!(gameloop.get_frame_num(), 1);
    }
}
