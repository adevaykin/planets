use super::sun;
use std::time;
use std::string;
use super::time::Time;

pub struct World {
    world_time: Time,
    sun: sun::Sun,
}

impl World {
    pub fn new() -> Self {
        println!("World is created!");
        World {
            world_time: Time::new(),
            sun: sun::Sun::new(),
        }
    }

    pub fn update(&mut self, prev_frame_time: time::Duration) {
        self.world_time.update(prev_frame_time);
        self.sun.update(self.world_time.get_time_since_update());
        
    }

    pub fn get_description_string(&self) -> string::String {        
        self.sun.get_description_string()
    }
}
