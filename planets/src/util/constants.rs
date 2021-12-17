use ash::vk::make_api_version;

pub const APPLICATION_VERSION: u32 = make_api_version(0, 1, 0, 0);
pub const ENGINE_VERSION: u32 = make_api_version(0, 1, 0, 0);
pub const API_VERSION: u32 = make_api_version(0, 1, 2, 0);

pub const ENGINE_NAME: &'static str = "2.5B";
pub const WINDOW_TITLE: &'static str = "2.5B Initiative";
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub const SHADERS_DIR: &'static str = "shaders/bin";
