use serde::{Deserialize, Serialize};
use std::string;
use std::time;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
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
        self.age += time_delta;
    }

    /// Get sun age
    pub fn get_age(&self) -> time::Duration {
        self.age
    }

    /// Return sun age as a String
    pub fn get_description_string(&self) -> string::String {
        return String::from(format!("Sun age is {:?}", self.age));
    }
}
