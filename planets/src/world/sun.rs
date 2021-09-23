use std::time;
use std::string;

pub struct Sun {
    /// Sun age counter
    sun_age: time::Duration,
    
        
}

impl Sun {
    pub fn new() -> Self {
        Sun {
            /// Sun age initialisation - 0 ms            
            sun_age: time::Duration::from_millis(0),          
            
        }
    }  

    /// Update sun age 
    pub fn sun_age_updt(&mut self, get_prev_frame_time: time::Duration) {
        self.sun_age += get_prev_frame_time;      
    }

    /// Get sun age
    pub fn get_sun_age(&self) -> time::Duration {                
        self.sun_age
    }
    
    /// Report sun age (sun age convertation to String)
    pub fn report_sun_age(&mut self) -> string::String{        
        return String::from(format!("Sun age is {:?}", self.sun_age));            
         
    }


}

