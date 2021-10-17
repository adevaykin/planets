use std::cell::RefCell;
use std::rc::{Rc, Weak};

use ash::vk;

extern crate cgmath as cgm;
use cgmath::prelude::*;

use super::device::Device;
use super::pipeline::Pipeline;
use super::resources::ResourceManager;
use super::shader::Binding;
use crate::engine::camera::Camera;
use crate::engine::geometry::{Geometry, Vertex};
use crate::engine::lights::LightManager;
use crate::engine::material::Material;
use crate::engine::timer::Timer;
use crate::vulkan::array_ssbo::ArraySSBO;
use crate::vulkan::mem::BufferData;
use std::hash::{Hash, Hasher};

pub fn get_default_vertex_input_binding_description() -> vk::VertexInputBindingDescription {
    let descr = vk::VertexInputBindingDescription {
        binding: 0 as u32,
        stride: std::mem::size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    };

    descr
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

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ShaderVertexData {
    pub model: cgm::Matrix4<f32>,
}

impl Default for ShaderVertexData {
    fn default() -> ShaderVertexData {
        ShaderVertexData {
            model: cgm::Matrix4::identity(),
        }
    }
}

impl BufferData for ShaderVertexData {
    fn size(&self) -> usize {
        std::mem::size_of::<cgm::Matrix4<f32>>()
    }

    fn stride(&self) -> u32 {
        self.size() as u32
    }

    fn as_ptr(&self) -> *const u8 {
        self.model.as_ptr() as *const u8
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum DrawType {
    Opaque,
    Wireframe,
}

pub type DrawableMutRef = Rc<RefCell<Drawable>>;
type DrawableWeakMutRef = Weak<RefCell<Drawable>>;

pub struct Drawable {
    pub draw_type: DrawType,
    buffer: ArraySSBO<ShaderVertexData>,
    instances: Vec<DrawableInstanceMutRef>,
    geometry: Geometry,
    pub material: Material,
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
        &*self as *const Self == &*other as *const Self
    }
}

impl Eq for DrawableHash {}

impl Hash for DrawableHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(&self.drawable.borrow(), state)
    }
}

impl Drawable {
    pub fn new(
        resource_manager: &mut ResourceManager,
        draw_type: DrawType,
        geometry: Geometry,
        material: Material,
    ) -> Drawable {
        Drawable {
            draw_type,
            buffer: ArraySSBO::new(resource_manager, "Drawable"),
            instances: vec![],
            geometry,
            material,
        }
    }

    pub fn draw(
        &self,
        device: &mut Device,
        resource_manager: &mut ResourceManager,
        camera: &Camera,
        light_manager: &LightManager,
        cmd_buffer: &vk::CommandBuffer,
        pipeline: &Pipeline,
    ) {
        if self.instances.is_empty() {
            return;
        }

        let descriptor_set = self.prepare_descriptor_set(
            device,
            resource_manager,
            pipeline,
            camera,
            light_manager,
        );
        let vertex_buffers = [self.geometry.vertex_buffer.borrow().buffer];
        let offsets = [0 as u64];

        unsafe {
            device.logical_device.cmd_bind_descriptor_sets(
                *cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                0,
                &[descriptor_set],
                &[],
            );

            device.logical_device.cmd_bind_vertex_buffers(
                *cmd_buffer,
                0,
                &vertex_buffers,
                &offsets,
            );
            device.logical_device.cmd_bind_index_buffer(
                *cmd_buffer,
                self.geometry.index_buffer.borrow().buffer,
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

    fn prepare_descriptor_set(
        &self,
        device: &mut Device,
        resource_manager: &mut ResourceManager,
        pipeline: &Pipeline,
        camera: &Camera,
        light_manager: &LightManager,
    ) -> vk::DescriptorSet {
        let descriptor_set = resource_manager
            .descriptor_set_manager
            .allocate_descriptor_set(device, &pipeline.descriptor_set_layout);

        let camera_buffer_info = vk::DescriptorBufferInfo {
            buffer: camera.ubo.buffer.borrow().buffer,
            range: camera.ubo.buffer.borrow().size as u64,
            ..Default::default()
        };

        let lights_buffer_info = vk::DescriptorBufferInfo {
            buffer: light_manager.ubo.buffer.borrow().buffer,
            range: light_manager.ubo.buffer.borrow().size as u64,
            ..Default::default()
        };

        let ssbo = self.buffer.gpu_buffer.borrow();
        let ssbo_info = vk::DescriptorBufferInfo {
            buffer: ssbo.buffer,
            range: ssbo.size as u64,
            ..Default::default()
        };

        let image_info = vk::DescriptorImageInfo {
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            image_view: self.material.albedo_map.as_ref().unwrap().views[0],
            sampler: self.material.albedo_map.as_ref().unwrap().sampler.sampler,
        };

        let descr_set_writes = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: 0,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &ssbo_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: Binding::Lights as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &lights_buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: Binding::Camera as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &camera_buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: 2,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                p_image_info: &image_info,
                ..Default::default()
            },
        ];

        unsafe {
            device
                .logical_device
                .update_descriptor_sets(&descr_set_writes, &[]);
        }

        descriptor_set
    }
}

pub type DrawableInstanceMutRef = Rc<RefCell<DrawableInstance>>;

pub struct DrawableInstance {
    pub drawable: DrawableWeakMutRef,
    instance_id: u64,
    data: ShaderVertexData,
}

impl DrawableInstance {
    pub fn update(&mut self, device: &Device, transform: &cgm::Matrix4<f32>) {
        self.data.model = *transform;
        self.update_buffer(device);
    }

    pub fn destroy(&mut self) {
        match self.drawable.upgrade() {
            Some(x) => {
                x.borrow_mut().remove_instance(self.instance_id);
            }
            None => log::error!("Failed to upgrade weak ref to parent Drawable for destroy()!"),
        };
    }

    fn update_buffer(&mut self, device: &Device) {
        match self.drawable.upgrade() {
            Some(x) => {
                x.borrow_mut().buffer.update_at(device, self.instance_id, &self.data);
            }
            None => {
                log::error!("Failed to upgrade weak ref to parent Drawable for update_buffer()!")
            }
        };
    }

    fn new(drawable: DrawableWeakMutRef, instance_id: u64) -> DrawableInstance {
        let data = ShaderVertexData::default();
        DrawableInstance {
            drawable,
            instance_id,
            data,
        }
    }

    fn update_instance_id(&mut self, new_id: u64) {
        self.instance_id = new_id;
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
        resource_manager: &mut ResourceManager,
        camera: &Camera,
        timer: &Timer,
        cmd_buffer: vk::CommandBuffer,
        pipeline: &Pipeline,
    ) {
        let descriptor_set = self.prepare_descriptor_set(
            device,
            resource_manager,
            camera,
            pipeline,
            timer,
        );
        let vertex_buffers = [self.geometry.vertex_buffer.borrow().buffer];
        let offsets = [0 as u64];

        unsafe {
            device.logical_device.cmd_bind_descriptor_sets(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.layout,
                0,
                &[descriptor_set],
                &[],
            );

            device.logical_device.cmd_bind_vertex_buffers(
                cmd_buffer,
                0,
                &vertex_buffers,
                &offsets,
            );
            device.logical_device.cmd_bind_index_buffer(
                cmd_buffer,
                self.geometry.index_buffer.borrow().buffer,
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

    // TODO: write a generic descriptor set preparation method to use everywhere*
    fn prepare_descriptor_set(
        &self,
        device: &Device,
        resource_manager: &mut ResourceManager,
        camera: &Camera,
        pipeline: &Pipeline,
        timer: &Timer,
    ) -> vk::DescriptorSet {
        let descriptor_set = resource_manager
            .descriptor_set_manager
            .allocate_descriptor_set(device, &pipeline.descriptor_set_layout);

        let timer_buffer_info = vk::DescriptorBufferInfo {
            buffer: timer.ubo.buffer.borrow().buffer,
            range: timer.ubo.buffer.borrow().size as u64,
            ..Default::default()
        };

        let camera_buffer_info = vk::DescriptorBufferInfo {
            buffer: camera.ubo.buffer.borrow().buffer,
            range: camera.ubo.buffer.borrow().size as u64,
            ..Default::default()
        };

        let descr_set_writes = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: Binding::Timer as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &timer_buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: Binding::Camera as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &camera_buffer_info,
                ..Default::default()
            },
        ];

        unsafe {
            device
                .logical_device
                .update_descriptor_sets(&descr_set_writes, &[]);
        }

        descriptor_set
    }
}
