use std::cell::RefCell;
use std::rc::{Rc, Weak};

use ash::vk;

extern crate cgmath as cgm;

use super::device::Device;
use super::resources::manager::ResourceManager;
use crate::engine::geometry::{Geometry, Vertex};
use crate::engine::material::Material;
use std::hash::{Hash, Hasher};

pub fn get_default_vertex_input_binding_description() -> vk::VertexInputBindingDescription {
    vk::VertexInputBindingDescription {
        binding: 0,
        stride: std::mem::size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }
}

pub fn get_default_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
    vec![
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 1,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: std::mem::size_of::<cgm::Vector3<f32>>() as u32,
        },
        vk::VertexInputAttributeDescription {
            binding: 0,
            location: 2,
            format: vk::Format::R32G32_SFLOAT,
            offset: 2 * std::mem::size_of::<cgm::Vector3<f32>>() as u32,
        },
    ]
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DrawType {
    Opaque,
}

pub type DrawableMutRef = Rc<RefCell<Drawable>>;
type DrawableWeakMutRef = Weak<RefCell<Drawable>>;

pub struct Drawable {
    pub draw_type: DrawType,
    instances: Vec<DrawableInstanceMutRef>,
    geometry: Geometry,
    pub material: Material,
}

impl Drawable {
    pub fn new(
        draw_type: DrawType,
        geometry: Geometry,
        material: Material,
    ) -> Drawable {
        Drawable {
            draw_type,
            instances: vec![],
            geometry,
            material,
        }
    }

    pub fn write_draw_commands(&self, device: &Device, cmd_buffer: &vk::CommandBuffer) {
        if self.instances.is_empty() {
            return;
        }

        let vertex_buffers = [self.geometry.vertex_buffer.borrow().get_vk_buffer()];
        let index_buffer = self.geometry.index_buffer.borrow().get_vk_buffer();
        let offsets = [0];

        unsafe {
            device.logical_device.cmd_bind_vertex_buffers(
                *cmd_buffer,
                0,
                &vertex_buffers,
                &offsets,
            );
            device.logical_device.cmd_bind_index_buffer(
                *cmd_buffer,
                index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            device.logical_device.cmd_draw_indexed(
                *cmd_buffer,
                self.geometry.indices.len() as u32,
                self.instances.len() as u32,
                0,
                0,
                0,
            );
        }
    }

    pub fn get_geometry(&self) -> &Geometry {
        &self.geometry
    }

    pub fn create_instance(drawable: &DrawableMutRef) -> DrawableInstanceMutRef {
        let instance = DrawableInstance::new(
            Rc::downgrade(drawable),
            drawable.borrow().instances.len() as u64,
        );
        let instance = Rc::new(RefCell::new(instance));
        drawable.borrow_mut().instances.push(Rc::clone(&instance));

        instance
    }

    fn remove_instance(&mut self, instance_id: u64) {
        self.instances.remove(instance_id as usize);
        for i in instance_id as usize..self.instances.len() {
            self.instances[i].borrow_mut().instance_id = i as u64;
        }
    }
}

pub struct DrawableHash {
    pub drawable: DrawableMutRef,
}

impl DrawableHash {
    pub fn new(drawable: &DrawableMutRef) -> Self {
        DrawableHash {
            drawable: Rc::clone(drawable),
        }
    }
}

impl PartialEq for DrawableHash {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for DrawableHash {}

impl Hash for DrawableHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(&self.drawable.borrow(), state)
    }
}

pub type DrawableInstanceMutRef = Rc<RefCell<DrawableInstance>>;

pub struct DrawableInstance {
    pub drawable: DrawableWeakMutRef,
    instance_id: u64,
}

impl DrawableInstance {
    pub fn destroy(&mut self) {
        match self.drawable.upgrade() {
            Some(x) => {
                x.borrow_mut().remove_instance(self.instance_id);
            }
            None => log::error!("Failed to upgrade weak ref to parent Drawable for destroy()!"),
        };
    }

    fn new(drawable: DrawableWeakMutRef, instance_id: u64) -> DrawableInstance {
        DrawableInstance {
            drawable,
            instance_id,
        }
    }

    pub fn get_instance_id(&self) -> u64 {
        self.instance_id
    }
}

pub struct FullScreenDrawable {
    geometry: Geometry,
}

impl FullScreenDrawable {
    pub fn new(resource_manager: &mut ResourceManager) -> FullScreenDrawable {
        let geometry = Geometry::quad(resource_manager);
        FullScreenDrawable { geometry }
    }

    pub fn draw(
        &self,
        device: &Device,
        cmd_buffer: vk::CommandBuffer,
    ) {
        let offsets = [0];

        let vertex_buffers = [self.geometry.vertex_buffer.borrow().get_vk_buffer()];
        let index_buffer = self.geometry.index_buffer.borrow().get_vk_buffer();

        unsafe {
            device
                .logical_device
                .cmd_bind_vertex_buffers(cmd_buffer, 0, &vertex_buffers, &offsets);
            device.logical_device.cmd_bind_index_buffer(
                cmd_buffer,
                index_buffer,
                0,
                vk::IndexType::UINT32,
            );

            device.logical_device.cmd_draw_indexed(
                cmd_buffer,
                self.geometry.indices.len() as u32,
                1,
                0,
                0,
                0,
            );
        }
    }
}
