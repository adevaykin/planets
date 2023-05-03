use ash::vk;

extern crate cgmath as cgm;
use cgmath::prelude::*;

use crate::vulkan::mem::{AllocatedBufferMutRef, VecBufferData};
use crate::vulkan::resources::manager::ResourceManager;

#[derive(Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: cgm::Vector3<f32>,
    pub normal: cgm::Vector3<f32>,
    pub uv: cgm::Vector2<f32>,
}

impl Vertex {
    #[cfg(debug_assertions)]
    pub fn from_position(x: f32, y: f32, z: f32) -> Vertex {
        Vertex {
            position: cgm::Vector3::new(x, y, z),
            normal: cgm::Vector3::zero(),
            uv: cgm::Vector2::zero(),
        }
    }
}

pub struct Geometry {
    pub vertices: Vec<Vertex>,
    pub vertex_buffer: AllocatedBufferMutRef,
    pub indices: Vec<u32>,
    pub index_buffer: AllocatedBufferMutRef,
}

impl Geometry {
    pub fn new(
        resource_manager: &mut ResourceManager,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Geometry {
        let vertex_data = VecBufferData::new(&vertices);
        let vertex_buffer = resource_manager.buffer_with_staging(
            &vertex_data,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR, // TODO: don't set RT usage if RT is inactive
            "Geometry::Vertex",
        );

        let index_data = VecBufferData::new(&indices);
        let index_buffer = resource_manager.buffer_with_staging(
            &index_data,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR, // TODO: don't set RT usage if RT is inactive
            "Geometry::Index",
        );

        Geometry {
            vertices,
            vertex_buffer,
            indices,
            index_buffer,
        }
    }

    #[allow(dead_code)]
    pub fn quad(resource_manager: &mut ResourceManager) -> Geometry {
        let triangle_verts = vec![
            Vertex {
                position: cgm::Vector3::new(-1.0, -1.0, 0.0),
                normal: cgm::Vector3::new(1.0, 0.0, 0.0),
                uv: cgm::Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: cgm::Vector3::new(1.0, -1.0, 0.0),
                normal: cgm::Vector3::new(0.0, 1.0, 0.0),
                uv: cgm::Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: cgm::Vector3::new(1.0, 1.0, 0.0),
                normal: cgm::Vector3::new(0.0, 0.0, 1.0),
                uv: cgm::Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: cgm::Vector3::new(-1.0, 1.0, 0.0),
                normal: cgm::Vector3::new(1.0, 1.0, 0.0),
                uv: cgm::Vector2::new(0.0, 1.0),
            },
        ];
        let triangle_index = vec![0, 2, 1, 0, 3, 2];

        Geometry::new(resource_manager, triangle_verts, triangle_index)
    }
}
