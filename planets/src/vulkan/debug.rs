use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;

use ash::extensions::ext::DebugUtils;
use ash::vk;

use super::device::Device;

pub trait DebugResource {
    fn get_type(&self) -> vk::ObjectType;
    fn get_handle(&self) -> u64;
    fn get_label(&self) -> &String;
}

pub struct Region {
    cmd_buffer: vk::CommandBuffer,
    debug_utils: DebugUtils,
}

impl Region {
    #[cfg(debug_assertions)]
    pub fn new(device: &Device, label: &str) -> Option<Region> {
        let debug_utils = DebugUtils::new(&device.entry, &device.instance.instance);
        let name = std::ffi::CString::new(label).unwrap();
        let label = vk::DebugUtilsLabelEXT {
            p_label_name: name.as_ptr(),
            ..Default::default()
        };
        let cmd_buffer = device.get_command_buffer();
        unsafe { debug_utils.cmd_begin_debug_utils_label(cmd_buffer, &label) };

        Some(Region {
            cmd_buffer,
            debug_utils,
        })
    }

    #[cfg(not(debug_assertions))]
    pub fn new(_device: &Device, _cmd_buffer: vk::CommandBuffer, _label: &str) -> Option<Region> {
        None
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        unsafe { self.debug_utils.cmd_end_debug_utils_label(self.cmd_buffer) };
    }
}

pub struct Object {}

impl Object {
    #[cfg(debug_assertions)]
    pub fn label(device: &Device, resource: &dyn DebugResource) {
        let full_label = match resource.get_type() {
            vk::ObjectType::BUFFER => String::from("Buffer:") + resource.get_label(),
            vk::ObjectType::DEVICE_MEMORY => String::from("Memory:") + resource.get_label(),
            vk::ObjectType::DESCRIPTOR_SET => String::from("DescriptorSet:") + resource.get_label(),
            vk::ObjectType::FRAMEBUFFER => String::from("Framebuffer:") + resource.get_label(),
            vk::ObjectType::FENCE => String::from("Fence:") + resource.get_label(),
            vk::ObjectType::IMAGE => String::from("Image:") + resource.get_label(),
            vk::ObjectType::IMAGE_VIEW => String::from("ImageView:") + resource.get_label(),
            vk::ObjectType::PIPELINE_LAYOUT => String::from("PipelineLayout:") + resource.get_label(),
            vk::ObjectType::PIPELINE => String::from("Pipeline:") + resource.get_label(),
            vk::ObjectType::RENDER_PASS => String::from("RenderPass:") + resource.get_label(),
            vk::ObjectType::SAMPLER => String::from("Sampler:") + resource.get_label(),
            vk::ObjectType::SEMAPHORE => String::from("Semaphore:") + resource.get_label(),
            vk::ObjectType::SHADER_MODULE => String::from("Shader:") + resource.get_label(),
            _ => {
                log::warn!("Tried to set label for vk object of unknown type");
                String::from("UnknownType:") + resource.get_label()
            },
        };

        let marker = DebugUtils::new(&device.entry, &device.instance.instance);
        let name = std::ffi::CString::new(full_label).unwrap();
        let name_info = vk::DebugUtilsObjectNameInfoEXT {
            object_type: resource.get_type(),
            object_handle: resource.get_handle(),
            p_object_name: name.into_raw(),
            ..Default::default()
        };
        unsafe {
            marker
                .set_debug_utils_object_name(device.logical_device.handle(), &name_info)
                .unwrap()
        };
    }

    #[cfg(not(debug_assertions))]
    pub fn label(_device: &Device, _obj_type: vk::ObjectType, _ptr: u64, _label: &str) {}
}

pub fn create_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
        //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
        //| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}

pub unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::debug!("{}{:?}", types, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("{}{:?}", types, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!("{}{:?}", types, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!("{}{:?}", types, message),
        _ => log::debug!("{}{:?}", types, message),
    }

    vk::FALSE
}
