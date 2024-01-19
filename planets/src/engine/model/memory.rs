use alloc::rc::Rc;
use std::cell::RefCell;
use ash::vk;
use cgmath::Vector3;
use crate::engine::geometry::{Geometry, Index, Vertex};
use crate::engine::model::geometry::{OwnedGeometry, OwnedGeometryMutRef};
use crate::vulkan::mem::{AllocatedBufferMutRef, VecBufferData};
use crate::vulkan::resources::manager::ResourceManager;

struct ModelMemoryManager {
    is_dirty: bool,
    quad: OwnedGeometryMutRef,
    geometries: Vec<OwnedGeometryMutRef>,
    vertices: Vec<Vertex>,
    indices: Vec<Index>,
    vertex_buffer: AllocatedBufferMutRef,
    index_buffer: AllocatedBufferMutRef,
}

impl ModelMemoryManager {
    pub fn new(resource_manager: &mut ResourceManager) -> Self {
        let (vertices, indices) = Geometry::quad_data();
        let vertices_buf_data = VecBufferData::new(&vertices);
        let vertex_buffer = resource_manager.buffer_with_staging(
            &vertices_buf_data,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR, // TODO: don't set RT usage if RT is inactive
            "Geometry::Vertex",
        );

        let indices_buf_data = VecBufferData::new(&indices);
        let index_buffer = resource_manager.buffer_with_staging(
            &indices_buf_data,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR, // TODO: don't set RT usage if RT is inactive
            "Geometry::Index",
        );

        let quad = Rc::new(RefCell::new(OwnedGeometry {
            vertex_offset: 0,
            index_offset: 0,
            vertex_count: vertices.len(),
            index_count: indices.len(),
        }));

        ModelMemoryManager {
            is_dirty: true,
            quad: Rc::clone(&quad),
            geometries: vec![quad],
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn update(&mut self, resource_manager: &mut ResourceManager) {
        if !self.is_dirty {
            return;
        }

        let old_vtx_size = self.vertex_buffer.borrow().size;
        let old_idx_size = self.index_buffer.borrow().size;

        let vertices_buf_data = VecBufferData::new(&self.vertices);
        let vertex_buffer = resource_manager.buffer_with_staging(
            &vertices_buf_data,
            vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR, // TODO: don't set RT usage if RT is inactive
            "Geometry::Vertex",
        );

        let indices_buf_data = VecBufferData::new(&self.indices);
        let index_buffer = resource_manager.buffer_with_staging(
            &indices_buf_data,
            vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR, // TODO: don't set RT usage if RT is inactive
            "Geometry::Index",
        );

        log::debug!("Updating vertex buffer size from {} to {}", old_vtx_size, vertex_buffer.borrow().size);
        log::debug!("Updating index buffer size from {} to {}", old_idx_size, index_buffer.borrow().size);

        self.vertex_buffer = vertex_buffer;
        self.index_buffer = index_buffer;
    }

    pub fn get_quad(&self) -> &OwnedGeometryMutRef {
        &self.quad
    }

    pub fn create_geometry(&mut self, vertices: Vec<Vertex>, indices: Vec<Index>) -> OwnedGeometryMutRef {
        let geometry = Rc::new(RefCell::new(OwnedGeometry {
            vertex_offset: self.vertices.len(),
            index_offset: self.indices.len(),
            vertex_count: vertices.len(),
            index_count: indices.len(),
        }));

        self.vertices.extend(vertices);
        self.indices.extend(indices);
        self.geometries.push(geometry.clone());
        self.is_dirty = true;
        geometry
    }

    pub fn get_vertex_buffer(&self) -> &AllocatedBufferMutRef {
        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &AllocatedBufferMutRef {
        &self.index_buffer
    }
}