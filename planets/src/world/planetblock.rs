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

mod tests {
    /// Compare two identical objects serialisation
    #[test]
    fn test_planet_block_serialisation_same() {
        let planet_block_one = PlanetBlock::new();
        let planet_block_two = PlanetBlock::new();

        let serialized_planet_block_one = serde_json::to_string(&planet_block_one).unwrap();
        let serialized_planet_two = serde_json::to_string(&planet_block_two).unwrap();

        assert_eq!(serialized_planet_block_one == serialized_planet_two, true);
    }

    /// Compare initial object serialisation with updated object serialisation
    #[test]
    fn test_planet_block_serialisation_diff() {
        let planet_block_one = PlanetBlock::new();

        let mut planet_block_three = PlanetBlock::new();
        planet_block_three.update(time::Duration::from_millis(1));

        let serialized_planet_block_one = serde_json::to_string(&planet_block_one).unwrap();
        let serialized_planet_block_three = serde_json::to_string(&planet_block_three).unwrap();

        assert_eq!(
            serialized_planet_block_one == serialized_planet_block_three,
            false
        );
    }

    /// Compare deserialised object with initial one
    #[test]
    fn test_planet_block_serial_deserial() {
        let planet_block_one = PlanetBlock::new();

        let serialized_planet_block_one = serde_json::to_string(&planet_block_one).unwrap();
        let deserialised_planet_block_one: PlanetBlock =
            serde_json::from_str(&serialized_planet_block_one).unwrap();

        assert_eq!(planet_block_one, deserialised_planet_block_one);
    }
}
