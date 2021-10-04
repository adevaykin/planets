use super::planet;
use super::sun;
use super::time::Time;
use serde::{Deserialize, Serialize};
use std::string;
use std::time;
use super::super::system::serialize::Save;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct World {
    world_time: Time,
    sun: sun::Sun,
    planet: planet::Planet,
}

impl World {
    pub fn new() -> Self {
        println!("World is created!");
        World {
            world_time: Time::new(),
            sun: sun::Sun::new(),
            planet: planet::Planet::new(),
        }
    }

    pub fn update(&mut self, prev_frame_time: time::Duration) {
        self.world_time.update(prev_frame_time);
        self.sun.update(self.world_time.get_time_since_update());
        self.planet.update(self.world_time.get_time_since_update());
    }

    pub fn get_description_string(&self) -> string::String {
        self.sun.get_description_string()
    }
}

impl Save for World {
    fn get_serialized_data(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

mod tests {

    use super::World;
    use serde::{Deserialize, Serialize};
    use std::cmp::Eq;
    use std::time;

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
        world_three.update(time::Duration::from_millis(1));

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
