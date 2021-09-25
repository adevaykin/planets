use std::time;
use std::string;

pub struct Sun {
    /// Sun age counter
    age: time::Duration,        
}

impl Sun {
    pub fn new() -> Self {
        Sun {
            /// Sun age initialisation - 0 ms            
            age: time::Duration::from_millis(0),
            
        }
    }  

    /// Update sun age 
    pub fn update(&mut self, time_delta: time::Duration) {
        self.sun_age += time_delta;
    }

    /// Get sun age
    pub fn get_sun_age(&self) -> time::Duration {                
        self.age
    }
    
    /// Return sun age as a String
    pub fn get_description_string (&self) -> string::String{        
        return String::from(format!("Sun age is {:?}", self.age));        
         
    }
}

