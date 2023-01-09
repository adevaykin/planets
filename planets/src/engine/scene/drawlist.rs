use std::rc::Rc;

use crate::engine::scene::graph::SceneGraph;
use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::drawable::{DrawType, DrawableHash};
use ash::vk;
use std::cell::RefCell;
use std::collections::HashSet;

pub type DrawListMutRef = Rc<RefCell<DrawList>>;

pub struct DrawList {
    device: DeviceMutRef,
    drawables: HashSet<DrawableHash>,
}

impl DrawList {
    pub fn new_mut_ref(device: &DeviceMutRef) -> DrawListMutRef {
        Rc::new(RefCell::new(DrawList::new(device)))
    }

    fn new(device: &DeviceMutRef) -> Self {
        DrawList {
            device: Rc::clone(device),
            drawables: HashSet::new(),
        }
    }

    pub fn add_drawables(&mut self, drawables: HashSet<DrawableHash>) {
        self.drawables.extend(drawables);
    }

    pub fn write_draw_commands(&self, draw_type: DrawType, cmd_buffer: &vk::CommandBuffer,) {
        let mut device = self.device.borrow();

        for d in &self.drawables {
            let d_ref = d.drawable.borrow();
            if d_ref.draw_type == draw_type {
                d_ref.write_draw_commands(&device, cmd_buffer);
            }
        }
    }

    pub fn end_frame(&mut self) {
        self.drawables.clear();
    }
}
