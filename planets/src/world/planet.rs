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
