use crate::engine::renderpass::{RenderPass};
use crate::engine::viewport::Viewport;
use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::resources::ResourceManagerMutRef;
use ash::vk;
use std::rc::Rc;
use crate::vulkan::image::image::Image;

pub struct Renderer {
    device: DeviceMutRef,
}

impl Renderer {
    pub fn new(
        device: &DeviceMutRef,
        resource_manager: &ResourceManagerMutRef,
        viewport: &Viewport,
    ) -> Self {
        Renderer {
            device: Rc::clone(device),
        }
    }

    pub fn render(&mut self) {
        let cmd_buffer = self.device.borrow().get_command_buffer();
        self.begin_frame(cmd_buffer);
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

    pub fn blit_result(&self, src_image: &mut Image, dst_image: &mut Image) {
        let device_ref = self.device.borrow();
        let cmd_buffer = device_ref.get_command_buffer();
        let src_offsets = [
            vk::Offset3D { x: 0, y: 0, z: 0 },
            vk::Offset3D {
                x: src_image.get_width() as i32,
                y: src_image.get_height() as i32,
                z: 1,
            },
        ];
        let dst_offsets = src_offsets;
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

        device_ref.transition_layout(dst_image, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

        unsafe {
            device_ref.logical_device.cmd_blit_image(
                cmd_buffer,
                src_image.get_image(),
                src_image.get_layout(),
                dst_image.get_image(),
                dst_image.get_layout(),
                &regions,
                vk::Filter::NEAREST,
            );
        }

        device_ref.transition_layout(dst_image, vk::ImageLayout::PRESENT_SRC_KHR);

        unsafe {
            device_ref
                .logical_device
                .end_command_buffer(cmd_buffer)
                .expect("Failed to end command buffer");
        }
    }
}
