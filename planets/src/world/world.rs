use super::super::system::serialize::Save;
use super::time::Time;
use serde::{Deserialize, Serialize};
use std::time;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct World {
    world_time: Time,
}

impl World {
    pub fn new() -> Self {
        println!("World is created!");
        World {
            world_time: Time::new(),
        }
    }

    pub fn update(&mut self, prev_frame_time: time::Duration) {
        self.world_time.update(prev_frame_time);
    }
}

impl Save for World {
    fn get_serialized_data(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::world::world::World;
    use std::time::Duration;

    /// Compare two identical objects serialisation
    #[test]
    fn test_world_serialisation_same() {
        let world_one = World::new();
        let world_two = World::new();

        let serialized_world_one = serde_json::to_string(&world_one).unwrap();
        let serialized_world_two = serde_json::to_string(&world_two).unwrap();

        assert_eq!(serialized_world_one == serialized_world_two, true);
    }

    /// Compare initial object serialisation with updated object serialisation
    #[test]
    fn test_sun_serialisation_diff() {
        let world_one = World::new();

        let mut world_three = World::new();
        world_three.update(Duration::from_millis(1));

        let serialized_world_one = serde_json::to_string(&world_one).unwrap();
        let serialized_world_three = serde_json::to_string(&world_three).unwrap();

        assert_eq!(serialized_world_one == serialized_world_three, false);
    }

    /// Compare deserialised object with initial one
    #[test]
    fn test_world_serial_deserial() {
        let world_one = World::new();

        let serialized_world_one = serde_json::to_string(&world_one).unwrap();
        let deserialised_world_one: World = serde_json::from_str(&serialized_world_one).unwrap();

        assert_eq!(world_one, deserialised_world_one);
    }
}
