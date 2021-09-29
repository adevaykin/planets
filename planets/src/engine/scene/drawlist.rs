use std::rc::Rc;

use crate::vulkan::drawable::{DrawType, DrawableHash};
use crate::vulkan::device::{MAX_FRAMES_IN_FLIGHT, DeviceMutRef};
use crate::vulkan::resources::{ResourceManagerMutRef};
use crate::engine::camera::Camera;
use crate::engine::lights::LightManager;
use ash::vk;
use crate::vulkan::pipeline::Pipeline;
use std::cell::RefCell;
use crate::engine::scene::graph::SceneGraph;
use std::collections::HashSet;

pub type DrawListMutRef = Rc<RefCell<DrawList>>;

pub struct DrawList {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    drawables: [HashSet<DrawableHash>; MAX_FRAMES_IN_FLIGHT],
}

impl DrawList {
    pub fn new(device: &DeviceMutRef, resource_manager: &ResourceManagerMutRef) -> Self {
        let drawables = [
            HashSet::new(),
            HashSet::new(),
        ];

        DrawList { device: Rc::clone(device), resource_manager: Rc::clone(resource_manager), drawables }
    }

    pub fn cull(&mut self, frame_num: usize, scene: &SceneGraph) {
        self.drawables[frame_num] = scene.cull();
    }

    pub fn draw(&self, frame_num: usize, draw_type: DrawType, camera: &Camera, light_manager: &LightManager,
                cmd_buffer: &vk::CommandBuffer, pipeline: &Pipeline) {
        let mut device = self.device.borrow_mut();
        let mut resource_manager = self.resource_manager.borrow_mut();

        for d in &self.drawables[frame_num] {
            let d_ref = d.drawable.borrow();
            if matches!(d_ref.draw_type, draw_type) {
                d_ref.draw(&mut *device,
                           &mut *resource_manager,
                           camera,
                           light_manager,
                           frame_num,
                           cmd_buffer,
                           pipeline);
            }
        }
    }

    pub fn end_frame(&mut self, frame_num: usize) {
        self.drawables[frame_num].clear();
    }
}