use std::cell::RefCell;
use std::rc::Rc;
use ash::vk::BufferUsageFlags;

use cgmath as cgm;
use cgmath::prelude::*;

use crate::vulkan::device::Device;
use crate::vulkan::mem::{AllocatedBufferMutRef, VecBufferData};
use crate::vulkan::resources::ResourceManager;

const MAX_LIGHTS: usize = 64;

pub type LightManagerMutRef = Rc<RefCell<LightManager>>;

#[derive(Clone)]
#[repr(C)]
struct LightBlock {
    position: cgm::Vector4<f32>,
    color: cgm::Vector4<f32>,
    is_active_radius_padding: cgm::Vector4<f32>,
}

impl LightBlock {
    pub fn new(position: cgm::Vector3<f32>) -> LightBlock {
        LightBlock {
            position: cgm::Vector4::new(position.x, position.y, position.z, 1.0),
            color: cgm::Vector4::new(1.0, 1.0, 1.0, 1.0),
            is_active_radius_padding: cgm::Vector4::new(0.0, f32::MAX, 0.0, 0.0),
        }
    }
}

#[derive(Copy, Clone)]
pub enum LightType {
    Point,
}

#[derive(Clone)]
pub struct Light {
    light_manager: LightManagerMutRef,
    light_id: usize,

    pub light_type: LightType,
    pub position: cgm::Vector3<f32>,
    pub color: cgm::Vector3<f32>,
    pub radius: f32,
    pub is_active: bool,
}

impl Light {
    fn new(light_manager: &LightManagerMutRef, light_id: usize) -> Light {
        Light {
            light_manager: Rc::clone(light_manager),
            light_id,
            light_type: LightType::Point,
            position: cgm::Vector3::zero(),
            color: cgm::Vector3::new(1.0, 1.0, 1.0),
            radius: 100.0,
            is_active: true,
        }
    }

    pub fn apply(&mut self) {
        let mut light_mgr = self.light_manager.borrow_mut();
        let mut light_block = &mut light_mgr.light_blocks[self.light_id];
        light_block.position =
            cgm::Vector4::new(self.position.x, self.position.y, self.position.z, 1.0);
        light_block.color = cgm::Vector4::new(self.color.x, self.color.y, self.color.z, 1.0);
        light_block.is_active_radius_padding.x = if self.is_active { 1.0 } else { 0.0 };
        light_block.is_active_radius_padding.y = self.radius;
    }
}

impl Drop for Light {
    fn drop(&mut self) {
        self.light_manager.borrow_mut().light_blocks[self.light_id]
            .is_active_radius_padding
            .x = 0.0;
        self.light_manager.borrow_mut().used_lights[self.light_id] = false;
    }
}

pub struct LightManager {
    ssbo: AllocatedBufferMutRef,
    light_blocks: Vec<LightBlock>,
    used_lights: Vec<bool>,
}

impl LightManager {
    pub fn new(resource_manager: &mut ResourceManager) -> LightManager {
        let mut light_blocks = vec![];
        light_blocks.resize(MAX_LIGHTS, LightBlock::new(cgm::Vector3::zero()));

        let mut used_lights = vec![];
        used_lights.resize(MAX_LIGHTS, false);

        let ssbo_data = VecBufferData::new(&light_blocks);
        let ssbo = resource_manager
            .buffer_host_visible_coherent(&ssbo_data, BufferUsageFlags::STORAGE_BUFFER, "ModelData");

        LightManager {
            ssbo,
            light_blocks,
            used_lights,
        }
    }

    pub fn update(&mut self, device: &Device) {
        let data = VecBufferData::new(&self.light_blocks);
        self.ssbo.borrow().update_data(device, &data, 0);
    }

    pub fn create_light(light_manager: &LightManagerMutRef) -> Light {
        let mut light_mgr_ref = light_manager.borrow_mut();
        for i in 0..light_mgr_ref.used_lights.len() {
            if !light_mgr_ref.used_lights[i] {
                light_mgr_ref.used_lights[i] = true;
                return Light::new(light_manager, i);
            }
        }

        panic!("Maximum number of lights used");
    }

    pub fn get_ssbo(&self) -> &AllocatedBufferMutRef {
        &self.ssbo
    }
}
