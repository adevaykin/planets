use ash::vk::make_version;

pub const APPLICATION_VERSION: u32 = make_version(1, 0, 0);
pub const ENGINE_VERSION: u32 = make_version(1, 0, 0);
pub const API_VERSION: u32 = make_version(1, 2, 0);

pub const ENGINE_NAME: &'static str = "Vulkan Engine";
pub const WINDOW_TITLE: &'static str = "Rust Vulkan";
pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub const SHADERS_DIR: &'static str = "shaders/bin";