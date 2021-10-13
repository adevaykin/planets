use serde::{Deserialize, Serialize};
use std::string;
use std::time;

// Trait for PlantBlocks copy
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlanetBlock {
    age: time::Duration,
}

impl PlanetBlock {
    pub fn new() -> Self {
        PlanetBlock {
            age: time::Duration::from_millis(0),
        }
    }

    /// Update PlanetBlock age
    pub fn update(&mut self, time_delta: time::Duration) {
        self.age += time_delta;
    }

    /// Get PlanetBlock age
    pub fn get_age(&self) -> time::Duration {
        self.age
    }

    /// Return PlanetBlock age as a String
    pub fn get_description_string(&self) -> string::String {
        return String::from(format!("Planet block age is {:?}", self.age));
    }
}
