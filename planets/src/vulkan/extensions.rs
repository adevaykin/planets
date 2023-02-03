use ash::extensions::ext::DebugUtils;
use ash::vk;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;
#[cfg(target_os = "macos")]
use ash::extensions::ext::MetalSurface;
use ash::extensions::khr::Surface;
#[cfg(target_os = "macos")]
use ash::vk;


// required extension ------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn required_instance_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MetalSurface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
        vk::KhrPortabilityEnumerationFn::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
pub fn required_instance_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_instance_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}
// ------------------------------------------------------------------------

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
        // Ray tracing extensions
        ash::extensions::khr::RayTracingPipeline::name().as_ptr(),
        ash::extensions::khr::AccelerationStructure::name().as_ptr(),
        ash::extensions::khr::DeferredHostOperations::name().as_ptr(),
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