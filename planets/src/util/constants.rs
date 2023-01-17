use ash::vk::make_api_version;

pub const APPLICATION_VERSION: u32 = make_api_version(0, 1, 0, 0);
pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0, 0);
pub const API_VERSION: u32 = make_api_version(0, 1, 2, 0);

pub const ENGINE_NAME: &str = "Planets";
pub const WINDOW_TITLE: &str = "Ray Tracing 3000";
pub const WINDOW_WIDTH: u32 = 1024;
pub const WINDOW_HEIGHT: u32 = 768;

pub const SHADERS_DIR: &str = "shaders/bin";
