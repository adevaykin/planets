use std::time::Duration;

pub struct Time {
    since_creation: Duration,
    multiplier: u32, // Speedup of the game time passing compared to real world time
}

impl Time {
    pub fn new() -> Self {
        Time {
            since_creation: Duration::from_secs(0),
            multiplier: 1,
        }
    }

    pub fn update(&mut self, prev_frame_time: Duration) {
        self.since_creation += prev_frame_time * self.multiplier;
    }

    // Set world time multiplier to speed up the worl time flow
    pub fn set_multiplier(&mut self, multiplier: u32) {
        self.multiplier = multiplier;
    }

    // Get time passed since the world was created
    pub fn get_since_creation(&self) -> Duration {
        self.since_creation
    }
}

mod tests {
    use super::Time;
    use std::time::Duration;

    #[test]
    fn test_set_multiplier() {
        let mut time = Time::new();

        // Update with default multiplier 1
        time.update(Duration::from_secs(3));
        assert_eq!(time.get_since_creation(), Duration::from_secs(3));

        // Update with a set multiplier
        time.set_multiplier(2);
        time.update(Duration::from_secs(2));
        assert_eq!(time.get_since_creation(), Duration::from_secs(3 + 2*2));

        // Update with multiplier set back to 1
        time.set_multiplier(1);
        time.update(Duration::from_secs(42));
        assert_eq!(time.get_since_creation(), Duration::from_secs(3 + 2*2 + 42));
    }
}