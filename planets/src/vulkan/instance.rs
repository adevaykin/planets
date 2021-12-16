use ash::vk;

use std::ffi::CString;
use std::os::raw::c_void;
use std::ptr;

use super::debug;
use crate::util::constants::*;
use crate::util::helpers;
use crate::util::platforms;

pub struct VulkanInstance {
    pub instance: ash::Instance,
    debug_utils_loader: Option<ash::extensions::ext::DebugUtils>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl VulkanInstance {
    pub fn new(entry: &ash::Entry) -> VulkanInstance {
        if helpers::is_debug() {
            VulkanInstance::log_extensions(&entry);

            if !VulkanInstance::check_validation_layers_support(&entry) {
                log::error!("Cannot continue debug build without debug layers");
                panic!("Cannot continue debug build without debug layers")
            }
        }

        let instance = VulkanInstance::create_instance(&entry);
        let (debug_utils_loader, debug_messenger) =
            VulkanInstance::setup_debug_callback(&entry, &instance);
        VulkanInstance {
            instance,
            debug_utils_loader,
            debug_messenger,
        }
    }

    fn create_instance(entry: &ash::Entry) -> ash::Instance {
        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new(ENGINE_NAME).unwrap();
        let app_info = vk::ApplicationInfo {
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: APPLICATION_VERSION,
            p_engine_name: engine_name.as_ptr(),
            engine_version: ENGINE_VERSION,
            api_version: API_VERSION,
            ..Default::default()
        };

        let mut extension_names = platforms::required_extension_names();
        let mut debug_extensions = helpers::debug_instance_extension_names();
        extension_names.append(&mut debug_extensions);
        let validation_layer_names = helpers::required_validation_layer_names();
        let validation_layer_names: Vec<CString> = validation_layer_names
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let validation_layer_names: Vec<*const i8> = validation_layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let debug_messenger_create_info = debug::create_messenger_create_info();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: if helpers::is_debug() {
                &debug_messenger_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                    as *const c_void
            } else {
                ptr::null()
            },
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_layer_names: if helpers::is_debug() {
                validation_layer_names.as_ptr()
            } else {
                ptr::null()
            },
            enabled_layer_count: if helpers::is_debug() {
                validation_layer_names.len() as u32
            } else {
                0 as u32
            },
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
        };

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance")
        };

        instance
    }

    fn log_extensions(entry: &ash::Entry) {
        let extensions = entry
            .enumerate_instance_extension_properties()
            .expect("Failed to enumerate extensions");

        println!("Enabled extensions:");
        for ext in extensions {
            let name = helpers::vulkan_str_to_str(&ext.extension_name);
            println!("{}", name);
        }
    }

    fn check_validation_layers_support(entry: &ash::Entry) -> bool {
        let required_layers = helpers::required_validation_layer_names();
        let available_layers = entry
            .enumerate_instance_layer_properties()
            .expect("Unable to enumerate validation layers");
        for required_layer_name in &required_layers {
            let mut found = false;
            for layer in &available_layers {
                let name = helpers::vulkan_str_to_str(&layer.layer_name);
                if name == String::from(*required_layer_name) {
                    found = true;
                    break;
                }
            }
            if !found {
                println!(
                    "Didn't find required validation layer: {}",
                    required_layer_name
                );
                return false;
            }
        }

        true
    }

    #[cfg(debug_assertions)]
    fn setup_debug_callback(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> (
        Option<ash::extensions::ext::DebugUtils>,
        Option<vk::DebugUtilsMessengerEXT>,
    ) {
        let create_info = debug::create_messenger_create_info();
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&create_info, None)
                .expect("Debug Utils Callback creation failed")
        };

        (Some(debug_utils_loader), Some(utils_messenger))
    }

    #[cfg(not(debug_assertions))]
    fn setup_debug_callback(
        _entry: &ash::Entry,
        _instance: &ash::Instance,
    ) -> (
        Option<ash::extensions::ext::DebugUtils>,
        Option<vk::DebugUtilsMessengerEXT>,
    ) {
        (None, None)
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            // TODO: we probably still need to destroy those, but it leads to SEH exception now
            // if self.debug_utils_loader.is_some() {
            //     self.debug_utils_loader
            //         .as_ref()
            //         .unwrap()
            //         .destroy_debug_utils_messenger(self.debug_messenger.unwrap(), None);
            // }
            //self.instance.destroy_instance(None)
        }
    }
}
