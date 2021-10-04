use super::super::system::serialize::Save;
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

mod tests {

    use super::Sun;
    use serde::{Deserialize, Serialize};
    use std::cmp::Eq;
    use std::time;

    /// Compare two identical objects serialisation
    #[test]
    fn test_sun_serialisation_same() {
        let sun_one = Sun::new();
        let sun_two = Sun::new();

        let serialized_sun_one = serde_json::to_string(&sun_one).unwrap();
        let serialized_sun_two = serde_json::to_string(&sun_two).unwrap();

        assert_eq!(serialized_sun_one == serialized_sun_two, true);
    }

    /// Compare initial object serialisation with updated object serialisation
    #[test]
    fn test_sun_serialisation_diff() {
        let sun_one = Sun::new();

        let mut sun_three = Sun::new();
        sun_three.update(time::Duration::from_millis(1));

        let serialized_sun_one = serde_json::to_string(&sun_one).unwrap();
        let serialized_sun_three = serde_json::to_string(&sun_three).unwrap();

        assert_eq!(serialized_sun_one == serialized_sun_three, false);
    }

    /// Compare deserialised object with initial one
    #[test]
    fn test_sun_serial_deserial() {
        let sun_one = Sun::new();

        let serialized_sun_one = serde_json::to_string(&sun_one).unwrap();
        let deserialised_sun_one: Sun = serde_json::from_str(&serialized_sun_one).unwrap();
        assert_eq!(sun_one, deserialised_sun_one);
    }
}
