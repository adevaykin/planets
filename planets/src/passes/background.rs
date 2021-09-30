use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use crate::vulkan::drawable::{FullScreenDrawable};
use crate::vulkan::renderpass::RenderPass;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::device::{DeviceMutRef};
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::shader::{Binding,ShaderManagerMutRef};
use crate::vulkan::framebuffer::Framebuffer;
use crate::vulkan::resources::{ResourceManagerMutRef};
use crate::vulkan::debug;
use crate::util::helpers::{ViewportSize, SimpleViewportSize};
use crate::engine::timer::TimerMutRef;

use crate::engine::camera::CameraMutRef;

pub struct BackgroundPass {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    timer: TimerMutRef,
    camera: CameraMutRef,
    pipeline: Pipeline,
    pub render_pass: vk::RenderPass,
    label: String,
    framebuffers: Vec<Framebuffer>,
    drawable: FullScreenDrawable,
}

struct SubpassDefinition {
    #[allow(dead_code)]
    attachment_refs: Vec<vk::AttachmentReference>,
    descriptions: Vec<vk::SubpassDescription>,
}

impl BackgroundPass {
    pub fn new(device: &DeviceMutRef, resource_manager: &ResourceManagerMutRef, timer: &TimerMutRef, swapchain: &Swapchain, shader_manager: &ShaderManagerMutRef,
        width: u32, height: u32, camera: &CameraMutRef, label: &str) -> BackgroundPass {
        let attachments = BackgroundPass::create_attachments(swapchain.format);
        let subpass_def = BackgroundPass::create_subpass_def();

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT  | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE  | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        }];

        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: subpass_def.descriptions.len() as u32,
            p_subpasses: subpass_def.descriptions.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
            ..Default::default()
        };

        let render_pass = unsafe {
            device.borrow().logical_device.create_render_pass(&render_pass_create_info, None).expect("Could not create render pass")
        };

        let layout_bindings = vec![
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Timer as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Camera as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];
        let pipeline = Pipeline::build(&device, shader_manager, render_pass, "background", width, height)
            .with_layout_bindings(layout_bindings)
            .assamble();

        let mut framebuffers = vec![];
        for i in 0..swapchain.images.len() {
            // TODO: care should be taken to access the correct view ASAP we will have more than one view for attachment
            debug_assert!(swapchain.images[i].views.len() == 1);
            let attachment_views = vec![swapchain.images[i].views[0]];
            framebuffers.push(Framebuffer::new(device, swapchain, attachment_views, render_pass));
        }

        debug::Object::label(&device.borrow(), vk::ObjectType::RENDER_PASS, render_pass.as_raw(), label);

        let drawable = FullScreenDrawable::new(&mut *resource_manager.borrow_mut());
        BackgroundPass { device: Rc::clone(device), resource_manager: Rc::clone(resource_manager), timer: Rc::clone(timer), camera: Rc::clone(camera), pipeline, render_pass, label: String::from(label), framebuffers, drawable }
    }

    fn create_attachments(format: vk::Format) -> Vec<vk::AttachmentDescription> {
        let attachment_descrs = vec![
            vk::AttachmentDescription {
                format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            }
        ];

        attachment_descrs
    }

    fn create_subpass_def() -> SubpassDefinition {
        let attachment_refs = vec![
            vk::AttachmentReference {
                attachment: 0 as u32,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
        ];

        let descriptions = vec![vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: attachment_refs.len() as u32,
            p_color_attachments: attachment_refs.as_ptr(),
            ..Default::default()
        }];

        SubpassDefinition { attachment_refs, descriptions }
    }

    fn record_commands(&self, swapchain: &Swapchain, frame_num: usize, command_buffer: &vk::CommandBuffer) {
        let mut device = self.device.borrow_mut();
        let mut _debug_region = debug::Region::new(&device, *command_buffer, self.label.as_str());

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                }
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                }
            }
        ];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[frame_num].framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        unsafe {
            device.logical_device.cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device.logical_device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipelines[0]);

            let mut resource_manager_ref = self.resource_manager.borrow_mut();
            self.drawable.draw(&mut device, &mut resource_manager_ref, &self.camera.borrow(), &self.timer.borrow(), frame_num, command_buffer, &self.pipeline);

            device.logical_device.cmd_end_render_pass(*command_buffer);
        }
    }
}

impl RenderPass for BackgroundPass {
    fn draw_frame(&mut self, swapchain: &Swapchain, frame_num: usize, command_buffer: &vk::CommandBuffer) {
        self.record_commands(&swapchain, frame_num, command_buffer);
    }
}

impl Drop for BackgroundPass {
    fn drop(&mut self) {
        unsafe {
            self.device.borrow().logical_device.destroy_render_pass(self.render_pass, None);
        }
    }
}
