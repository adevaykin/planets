use std::ffi::CStr;

pub fn required_validation_layer_names() -> [&'static str; 1] {
    ["VK_LAYER_KHRONOS_validation"]
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
    let raw_string = unsafe { CStr::from_ptr(c_str_ptr) };

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
