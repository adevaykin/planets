use ash::vk::make_version;

pub const APPLICATION_VERSION: u32 = make_version(1, 0, 0);
pub const ENGINE_VERSION: u32 = make_version(1, 0, 0);
pub const API_VERSION: u32 = make_version(1, 2, 0);

pub const ENGINE_NAME: &'static str = "Planets";
pub const WINDOW_TITLE: &'static str = "Planets";

pub const SHADERS_DIR: &'static str = "shaders/bin";
