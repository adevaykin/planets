use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::ptr;
use std::rc::Rc;

use ash::vk;
use ash::vk::{Handle};

use super::debug;
use super::device::DeviceMutRef;
use crate::util;
use crate::util::constants;
use crate::vulkan::debug::DebugResource;
use crate::vulkan::device::Device;

pub type ShaderManagerMutRef = Rc<RefCell<ShaderManager>>;

pub enum Binding {
    ObjDescrs = 12,
    Models = 13,
    Lights = 14,
    Timer = 15,
    Camera = 16,
}

pub struct ShaderManager {
    device: DeviceMutRef,
    shaders: HashMap<String, Shader>,
}

pub struct Shader {
    device: DeviceMutRef,
    pub vertex_module: Option<ShaderModule>,
    pub fragment_module: Option<ShaderModule>,
    pub raygen_module: Option<ShaderModule>,
    pub chit_module: Option<ShaderModule>,
    pub miss_module: Option<ShaderModule>,
}

impl ShaderManager {
    pub fn new(device: &DeviceMutRef) -> ShaderManager {
        ShaderManager {
            device: Rc::clone(device),
            shaders: HashMap::new(),
        }
    }

    pub fn get_shader(&mut self, name: &str) -> &Shader {
        let existing_shader = self.shaders.get(name);
        if existing_shader.is_none() {
            let new_shader = self.load_shader(name);
            self.shaders.insert(String::from(name), new_shader);
        }

        self.shaders.get(name).unwrap()
    }

    fn load_shader(&self, name: &str) -> Shader {
        let path = Path::new(constants::SHADERS_DIR);

        Shader::new(&self.device, path, name)
    }
}

impl Shader {
    fn new(device: &DeviceMutRef, path: &Path, name: &str) -> Shader {
        let mut vert_filename = String::from(name);
        vert_filename.push_str(".vert.spv");
        let vertex_module = Self::load_from_file(&device.borrow(), path, &vert_filename);

        let mut frag_filename = String::from(name);
        frag_filename.push_str(".frag.spv");
        let fragment_module = Self::load_from_file(&device.borrow(), path, &frag_filename);

        let mut raygen_filename = String::from(name);
        raygen_filename.push_str(".rgen.spv");
        let raygen_module = Self::load_from_file(&device.borrow(), path, &raygen_filename);

        let mut chit_filename = String::from(name);
        chit_filename.push_str(".rchit.spv");
        let chit_module = Self::load_from_file(&device.borrow(), path, &chit_filename);

        let mut miss_filename = String::from(name);
        miss_filename.push_str(".rmiss.spv");
        let miss_module = Self::load_from_file(&device.borrow(), path, &miss_filename);

        Shader {
            device: Rc::clone(device),
            vertex_module,
            fragment_module,
            raygen_module,
            chit_module,
            miss_module,
        }
    }

    fn load_from_file(device: &Device, path: &Path, filename: &String) -> Option<ShaderModule> {
        let shader_path = path.join(Path::new(&filename));
        if shader_path.exists() {
            let vertex_data =
                util::fs::read_bin_file(shader_path.as_path()).expect("Could not load vertex shader");
            let module = ShaderModule {
                module: Shader::create_module(&device.logical_device, vertex_data),
                label: filename.clone()
            };
            debug::Object::label(device, &module);

            Some(module)
        } else {
            None
        }
    }

    fn create_module(device: &ash::Device, data: Vec<u8>) -> vk::ShaderModule {
        let shader_module_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: data.len(),
            p_code: data.as_ptr() as *const u32,
        };

        unsafe {
            device
                .create_shader_module(&shader_module_info, None)
                .expect("Failed to create shader module")
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            if let Some(module) = &self.vertex_module {
                self.device.borrow().logical_device.destroy_shader_module(
                    module.get_module(),
                    None,
                );
            }
            if let Some(module) = &self.fragment_module {
                self.device.borrow().logical_device.destroy_shader_module(
                    module.get_module(),
                    None,
                );
            }
        }
    }
}

pub struct ShaderModule {
    module: vk::ShaderModule,
    label: String,
}

impl ShaderModule {
    pub fn get_module(&self) -> vk::ShaderModule {
        self.module
    }
}

impl DebugResource for ShaderModule {
    fn get_type(&self) -> vk::ObjectType {
        vk::ObjectType::SHADER_MODULE
    }

    fn get_handle(&self) -> u64 {
        self.module.as_raw()
    }

    fn get_label(&self) -> &String {
        &self.label
    }
}
