use std::cell::RefCell;
use std::rc::Rc;

use cgmath as cgm;
use crate::util::constants::{WINDOW_HEIGHT, WINDOW_WIDTH};

use crate::vulkan::device::{Device, MAX_FRAMES_IN_FLIGHT};
use crate::vulkan::mem::StructBufferData;
use crate::vulkan::resources::manager::ResourceManager;
use crate::vulkan::uniform_buffer::UniformBufferObject;

pub type CameraMutRef = Rc<RefCell<Camera>>;
pub const UP: cgm::Vector3<f32> = cgm::Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
}; // TODO: move this constant to some kind of World from Camera

#[repr(C)]
pub struct CameraUBOInterface {
    pub view: cgm::Matrix4<f32>,
    pub proj: cgm::Matrix4<f32>,
    pub viewport_extent: cgm::Vector4<f32>,
}

pub struct Camera {
    viewport_size: cgm::Vector2<u32>,
    pub position: cgm::Point3<f32>,
    up: cgm::Vector3<f32>,
    pub aspect: f32,
    pub ubo_interface: CameraUBOInterface,
    ubo: Vec<UniformBufferObject>,
}

impl Camera {
    pub fn new(resource_manager: &mut ResourceManager) -> Camera {
        let position = cgm::Point3 {
            x: 0.0,
            y: 0.0,
            z: -2.0,
        };
        let up = UP;
        let aspect = 4.0 / 3.0;
        let mut ubo_interface = CameraUBOInterface {
            view: cgm::Matrix4::look_at_rh(position, cgm::Point3::new(0.0, 0.0, 0.0), up),
            proj: cgm::perspective(cgm::Deg(60.0), aspect, 0.1, 100.0),
            viewport_extent: cgm::Vector4 {
                x: WINDOW_WIDTH as f32,
                y: WINDOW_HEIGHT as f32,
                z: 0.0,
                w: 0.0,
            },
        };
        ubo_interface.proj[1][1] *= -1.0;

        let ubo_data = StructBufferData::new(&ubo_interface);
        let mut ubo = vec![];
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            ubo.push(UniformBufferObject::new_with_data(resource_manager, &ubo_data, format!("Camera{}", i).as_str()));
        }

        Camera {
            viewport_size: cgm::Vector2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            position,
            up,
            aspect,
            ubo_interface,
            ubo,
        }
    }

    pub fn update(&mut self, device: &Device, viewport_width: u32, viewport_height: u32) {
        let mut ubo_interface = CameraUBOInterface {
            view: cgm::Matrix4::look_at_rh(self.position, cgm::Point3::new(0.0, 0.0, 0.0), self.up),
            proj: cgm::perspective(cgm::Deg(45.0), self.aspect, 0.1, 100.0),
            viewport_extent: cgm::Vector4 {
                x: 0 as f32,
                y: 0 as f32,
                z: viewport_width as f32,
                w: viewport_height as f32,
            },
        };
        ubo_interface.proj[1][1] *= -1.0;

        let ubo_data = StructBufferData::new(&ubo_interface);
        self.ubo[device.get_image_idx()].buffer.borrow().update_data(device, &ubo_data, 0);
    }

    pub fn get_ubo(&self, image_idx: usize) -> &UniformBufferObject {
        &self.ubo[image_idx]
    }

    pub fn get_viewport_size(&self) -> &cgm::Vector2<u32> {
        &self.viewport_size
    }
}
