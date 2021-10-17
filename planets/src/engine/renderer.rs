use crate::engine::framegraph::{FrameGraph, RenderPass};
use crate::vulkan::device::DeviceMutRef;
use std::rc::Rc;
use ash::vk;

pub struct Renderer {
    device: DeviceMutRef,
    frame_graph: FrameGraph,
}

impl Renderer {
    pub fn new(device: &DeviceMutRef) -> Self {
        Renderer {
            device: Rc::clone(device),
            frame_graph: FrameGraph::new(),
        }
    }

    pub fn render(&mut self, frame_idx: usize) {
        let cmd_buffer = self.device.borrow().command_buffers[frame_idx];
        self.begin_frame(cmd_buffer);

        self.frame_graph.build();
        self.frame_graph.execute(cmd_buffer);

        self.end_frame(cmd_buffer);
    }

    pub fn add_pass(&mut self, render_pass: Box<dyn RenderPass>) {
        self.frame_graph.add_pass(render_pass);
    }

    fn begin_frame(&self, cmd_buffer: vk::CommandBuffer) {
        let logical_device = &self.device.borrow().logical_device;

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            ..Default::default()
        };
        unsafe { logical_device.reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::default()).expect("Failed to reset command buffer"); }
        unsafe { logical_device.begin_command_buffer(cmd_buffer, &begin_info).expect("Failed to begin command buffer"); }
    }

    fn end_frame(&self, cmd_buffer: vk::CommandBuffer) {
        unsafe { self.device.borrow().logical_device.end_command_buffer(cmd_buffer).expect("Failed to end command buffer"); }
    }
}