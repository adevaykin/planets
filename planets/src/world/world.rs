use super::sun;
use std::time;
use std::string;

pub struct World {
    sun: sun::Sun,       

}

impl World {
        pub fn new() -> Self{
        println!("World is created!");
        World {             
            sun: sun::Sun::new(),             
        }
    }

    /// Update the world objects
    pub fn update(&mut self, prev_frame_time: time::Duration) {
        self.sun.updt(prev_frame_time);

    }

    /// Return status of world objects as a string
    pub fn get_description_string(&mut self) -> string::String {        
        self.sun.get_description_string()

    }
}
