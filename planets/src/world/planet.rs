use super::planetblock;
use serde::{Deserialize, Serialize};

const BLOKS_ARRAY_SIZE: usize = 5;

// Array is declared for keeping 25 planet blocks.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Planet {
    planetblocks_array: [[planetblock::PlanetBlock; BLOKS_ARRAY_SIZE]; BLOKS_ARRAY_SIZE],
}

impl Planet {
    // Initialization of array with PlanetBlock objects
    pub fn new() -> Self {
        Planet {
            planetblocks_array: [[planetblock::PlanetBlock::new(); BLOKS_ARRAY_SIZE];
                BLOKS_ARRAY_SIZE],
        }
    }
    // Updating the planet blocks age
    pub fn update(&mut self, time_delta: std::time::Duration) {
        for i in 0..BLOKS_ARRAY_SIZE {
            for j in 0..BLOKS_ARRAY_SIZE {
                self.planetblocks_array[i][j].update(time_delta);

                // Printing of age of planet block
                println!("{} {} {:?}", i, j, self.planetblocks_array[i][j]);
            }
        }
    }
}

mod tests {

    use super::Planet;
    use serde::{Deserialize, Serialize};
    use std::cmp::Eq;
    use std::time;

    /// Compare two identical objects serialisation
    #[test]
    fn test_planet_serialisation_same() {
        let planet_one = Planet::new();
        let planet_two = Planet::new();

        let serialized_planet_one = serde_json::to_string(&planet_one).unwrap();
        let serialized_planet_two = serde_json::to_string(&planet_two).unwrap();

        assert_eq!(serialized_planet_one == serialized_planet_two, true);
    }

    /// Compare initial object serialisation with updated object serialisation
    #[test]
    fn test_planet_serialisation_diff() {
        let planet_one = Planet::new();

        let mut planet_three = Planet::new();
        planet_three.update(time::Duration::from_millis(1));

        let serialized_planet_one = serde_json::to_string(&planet_one).unwrap();
        let serialized_planet_three = serde_json::to_string(&planet_three).unwrap();

        assert_eq!(serialized_planet_one == serialized_planet_three, false);
    }

    /// Compare deserialised object with initial one
    #[test]
    fn test_planet_serial_deserial() {
        let planet_one = Planet::new();

        let serialized_planet_one = serde_json::to_string(&planet_one).unwrap();
        let deserialised_planet_one: Planet = serde_json::from_str(&serialized_planet_one).unwrap();

        assert_eq!(planet_one, deserialised_planet_one);
    }
}
