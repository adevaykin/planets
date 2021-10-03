use std::cell::RefCell;
use std::rc::Rc;

use cgmath as cgm;

use crate::util::helpers::ViewportSize;
use crate::vulkan::device::Device;
use crate::vulkan::mem::StructBufferData;
use crate::vulkan::resources::ResourceManager;
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
    pub position: cgm::Point3<f32>,
    up: cgm::Vector3<f32>,
    pub aspect: f32,
    pub ubo_interface: CameraUBOInterface,
    pub ubos: Vec<UniformBufferObject>,
}

impl Camera {
    pub fn new(resource_manager: &mut ResourceManager) -> Camera {
        let position = cgm::Point3 {
            x: 0.0,
            y: 0.0,
            z: -10.0,
        };
        let up = UP;
        let aspect = 4.0 / 3.0;
        let mut ubo_interface = CameraUBOInterface {
            view: cgm::Matrix4::look_at_rh(position, cgm::Point3::new(0.0, 0.0, 0.0), up),
            proj: cgm::perspective(cgm::Deg(45.0), aspect, 0.1, 100.0),
            viewport_extent: cgm::Vector4 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            },
        };
        ubo_interface.proj[1][1] *= -1.0;

        let ubo_data = StructBufferData::new(&ubo_interface);
        let ubos = vec![
            UniformBufferObject::new_with_data(resource_manager, &ubo_data, "Camera"),
            UniformBufferObject::new_with_data(resource_manager, &ubo_data, "Camera"),
        ];

        Camera {
            position,
            up,
            aspect,
            ubo_interface,
            ubos,
        }
    }

    pub fn update(&mut self, device: &Device, frame_num: usize, viewport_size: &dyn ViewportSize) {
        let mut ubo_interface = CameraUBOInterface {
            view: cgm::Matrix4::look_at_rh(self.position, cgm::Point3::new(0.0, 0.0, 0.0), self.up),
            proj: cgm::perspective(cgm::Deg(45.0), self.aspect, 0.1, 100.0),
            viewport_extent: cgm::Vector4 {
                x: viewport_size.get_size().offset_x,
                y: viewport_size.get_size().offset_y,
                z: viewport_size.get_size().width,
                w: viewport_size.get_size().height,
            },
        };
        ubo_interface.proj[1][1] *= -1.0;

        let ubo_data = StructBufferData::new(&ubo_interface);
        self.ubos[frame_num]
            .buffer
            .borrow()
            .update_data(device, &ubo_data, 0);
    }
}
