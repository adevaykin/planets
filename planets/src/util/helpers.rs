use std::ffi::CStr;
use ash::vk;

#[cfg(target_os = "macos")]
pub fn required_device_extension_names() -> Vec<*const i8> {
    vec![
        ash::extensions::khr::Swapchain::name().as_ptr(),
        vk::KhrPortabilitySubsetFn::name().as_ptr(),
    ]
}

#[cfg(target_os = "windows")]
pub fn required_device_extension_names() -> Vec<*const i8> {
    vec![
        ash::extensions::khr::Swapchain::name().as_ptr(),
    ]
}

#[cfg(target_os = "macos")]
pub fn get_instance_creation_flags() -> vk::InstanceCreateFlags {
    vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
}

#[cfg(target_os = "windows")]
pub fn get_instance_creation_flags() -> vk::InstanceCreateFlags {
    vk::InstanceCreateFlags::empty()
}

#[cfg(debug_assertions)]
pub fn debug_device_extension_names() -> Vec<*const i8> {
    vec![]
}

#[cfg(debug_assertions)]
pub fn debug_instance_extension_names() -> Vec<*const i8> {
    vec![ash::extensions::ext::DebugUtils::name().as_ptr()]
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
