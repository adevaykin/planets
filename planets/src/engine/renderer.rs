use crate::engine::framegraph::{FrameGraph, RenderPass};
use crate::engine::viewport::Viewport;
use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::image::Image;
use crate::vulkan::resources::ResourceManagerMutRef;
use ash::vk;
use std::rc::Rc;

pub struct Renderer {
    device: DeviceMutRef,
    frame_graph: FrameGraph,
}

impl Renderer {
    pub fn new(
        device: &DeviceMutRef,
        resource_manager: &ResourceManagerMutRef,
        viewport: &Viewport,
    ) -> Self {
        Renderer {
            device: Rc::clone(device),
            frame_graph: FrameGraph::new(resource_manager, viewport),
        }
    }

    pub fn render(&mut self, frame_idx: usize) {
        let cmd_buffer = self.device.borrow().command_buffers[frame_idx];
        self.begin_frame(cmd_buffer);

        self.frame_graph.build();
        self.frame_graph.execute(cmd_buffer);
    }

    pub fn add_pass(&mut self, render_pass: Box<dyn RenderPass>) {
        self.frame_graph.add_pass(render_pass);
    }

    pub fn begin_frame(&self, cmd_buffer: vk::CommandBuffer) {
        let logical_device = &self.device.borrow().logical_device;

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            ..Default::default()
        };
        unsafe {
            logical_device
                .reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::default())
                .expect("Failed to reset command buffer");
        }
        unsafe {
            logical_device
                .begin_command_buffer(cmd_buffer, &begin_info)
                .expect("Failed to begin command buffer");
        }
    }

    pub fn blit_result(&self, frame_idx: usize, dst_image: &mut Image) {
        let render_result = self.frame_graph.get_result();
        let render_image = render_result.borrow();

        let logical_device = &self.device.borrow().logical_device;
        let cmd_buffer = self.device.borrow().command_buffers[frame_idx];
        let src_offsets = [
            vk::Offset3D { x: 0, y: 0, z: 0 },
            vk::Offset3D {
                x: render_image.get_width() as i32,
                y: render_image.get_height() as i32,
                z: 1,
            },
        ];
        let dst_offsets = [
            vk::Offset3D { x: 0, y: 0, z: 0 },
            vk::Offset3D {
                x: dst_image.get_width() as i32,
                y: dst_image.get_height() as i32,
                z: 1,
            },
        ];
        let regions = [vk::ImageBlit {
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_offsets,
            dst_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            dst_offsets,
        }];

        dst_image.transition_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL, cmd_buffer);

        unsafe {
            logical_device.cmd_blit_image(
                cmd_buffer,
                render_image.image,
                render_image.get_layout(),
                dst_image.image,
                dst_image.get_layout(),
                &regions,
                vk::Filter::NEAREST,
            );
        }

        dst_image.transition_layout(vk::ImageLayout::PRESENT_SRC_KHR, cmd_buffer);

        unsafe {
            self.device
                .borrow()
                .logical_device
                .end_command_buffer(cmd_buffer)
                .expect("Failed to end command buffer");
        }
    }
}
