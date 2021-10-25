use std::rc::Rc;

use crate::engine::camera::Camera;
use crate::engine::lights::LightManager;
use crate::engine::scene::graph::SceneGraph;
use crate::vulkan::device::{DeviceMutRef};
use crate::vulkan::drawable::{DrawType, DrawableHash};
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::resources::ResourceManagerMutRef;
use ash::vk;
use std::cell::RefCell;
use std::collections::HashSet;

pub type DrawListMutRef = Rc<RefCell<DrawList>>;

pub struct DrawList {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    drawables: HashSet<DrawableHash>,
}

impl DrawList {
    pub fn new(device: &DeviceMutRef, resource_manager: &ResourceManagerMutRef) -> Self {
        DrawList {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            drawables: HashSet::new(),
        }
    }

    pub fn cull(&mut self, scene: &SceneGraph) {
        self.drawables = scene.cull();
    }

    pub fn draw(
        &self,
        draw_type: DrawType,
        camera: &Camera,
        light_manager: &LightManager,
        cmd_buffer: &vk::CommandBuffer,
        pipeline: &Pipeline,
    ) {
        let mut device = self.device.borrow_mut();
        let mut resource_manager = self.resource_manager.borrow_mut();

        for d in &self.drawables {
            let mut d_ref = d.drawable.borrow_mut();
            if matches!(d_ref.draw_type, draw_type) {
                d_ref.draw(
                    &mut *device,
                    &mut *resource_manager,
                    camera,
                    light_manager,
                    cmd_buffer,
                    pipeline,
                );
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.drawables.clear();
    }
}
