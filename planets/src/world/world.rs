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

    pub fn update(&mut self, get_prev_frame_time: time::Duration) {
        self.sun.sun_age_updt(get_prev_frame_time);

    }

    pub fn report_world_status(&mut self) -> string::String {        
        self.sun.report_sun_age() 

    }

    




}
