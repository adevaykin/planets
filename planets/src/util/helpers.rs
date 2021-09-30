use std::ffi::CStr;

pub fn required_device_extension_names() -> Vec<*const i8> {
    vec![
        ash::extensions::khr::Swapchain::name().as_ptr()
    ]
}

#[cfg(debug_assertions)]
pub fn debug_device_extension_names() -> Vec<*const i8> {
    vec![
        
    ]
}

#[cfg(debug_assertions)]
pub fn debug_instance_extension_names() -> Vec<*const i8> {
    vec![
        ash::extensions::ext::DebugUtils::name().as_ptr(),
    ]
}

#[cfg(not(debug_assertions))]
pub fn debug_device_extension_names() -> Vec<*const i8> {
    vec![]
}

#[cfg(not(debug_assertions))]
pub fn debug_instance_extension_names() -> Vec<*const i8> {
    vec![]
}

pub fn required_validation_layer_names() -> [&'static str; 1] {
    [ "VK_LAYER_KHRONOS_validation" ]
}

pub fn vulkan_str_to_str(c_string: &[i8; 256]) -> String {
    let raw_string = unsafe {
        let pointer = c_string.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

pub fn c_str_ptr_to_str(c_str_ptr: *const i8) -> String {
    let raw_string = unsafe {
        CStr::from_ptr(c_str_ptr)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

#[cfg(debug_assertions)]
pub fn is_debug() -> bool {
    true
}

#[cfg(not(debug_assertions))]
pub fn is_debug() -> bool {
    false
}

pub trait ViewportSize {
    fn get_size(&self) -> SimpleViewportSize;
}

#[derive(Copy,Clone)]
pub struct SimpleViewportSize {
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: f32,
    pub height: f32,
}

impl SimpleViewportSize {
    pub fn from_width_height(width: u32, height: u32) -> Self {
        SimpleViewportSize {
            offset_x: 0.0,
            offset_y: 0.0,
            width: width as f32,
            height: height as f32
        }
    }
}

impl ViewportSize for SimpleViewportSize {
    fn get_size(&self) -> SimpleViewportSize {
        *self
    }
}
