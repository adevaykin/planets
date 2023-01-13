// Code was taken directly from the Vulkan-Rust tutorial repo https://github.com/unknownue/vulkan-tutorial-rust

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;
#[cfg(target_os = "macos")]
use ash::extensions::ext::MetalSurface;
#[cfg(target_os = "macos")]
use ash::vk;


// required extension ------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MetalSurface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
        vk::KhrPortabilityEnumerationFn::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}
// ------------------------------------------------------------------------

// create surface ---------------------------------------------------------
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window: &winit::window::Window,
) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::ptr;
    use winit::platform::unix::WindowExtUnix;

    let x11_display = window.xlib_display().unwrap();
    let x11_window = window.xlib_window().unwrap();
    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };
    let xlib_surface_loader = XlibSurface::new(entry, instance);
    xlib_surface_loader.create_xlib_surface(&x11_create_info, None)
}
