use crate::world::world::World;
use std::fs;
use std::fs::File;
use std::io::Write;

pub trait Save {
    fn get_serialized_data(&self) -> String;
}

pub struct Saver {}

impl Saver {
    pub fn new() -> Self {
        Saver {}
    }
    pub fn save(&self, object_to_save: &dyn Save) {
        let saved_data = object_to_save.get_serialized_data();
        let mut writer = File::create("./savegame.json").unwrap();
        write!(writer, "{}", saved_data).expect("Failed to save world state to file.");
    }
}

pub struct Loader {}

impl Loader {
    pub fn new() -> Self {
        Loader {}
    }

    pub fn load(&self) -> World {
        let world_str = fs::read_to_string("./savegame.json").expect("Can't read the file");
        let world: World = serde_json::from_str(&world_str).unwrap();
        world
    }
}
