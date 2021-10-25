use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use crate::engine::timer::TimerMutRef;
use crate::vulkan::debug;
use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::drawable::FullScreenDrawable;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::resources::ResourceManagerMutRef;
use crate::vulkan::shader::{Binding, ShaderManagerMutRef};
use crate::engine::viewport::ViewportMutRef;

use crate::engine::camera::CameraMutRef;
use crate::engine::framegraph::{RenderPass, Attachment, AttachmentDirection, AttachmentSize};

pub struct BackgroundPass {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    timer: TimerMutRef,
    viewport: ViewportMutRef,
    camera: CameraMutRef,
    pipeline: Pipeline,
    pub render_pass: vk::RenderPass,
    label: &'static str,
    drawable: FullScreenDrawable,
    attachments: Vec<Attachment>,
}

struct SubpassDefinition {
    #[allow(dead_code)]
    attachment_refs: Vec<vk::AttachmentReference>,
    descriptions: Vec<vk::SubpassDescription>,
}

impl BackgroundPass {
    pub fn new(
        device: &DeviceMutRef,
        resource_manager: &ResourceManagerMutRef,
        timer: &TimerMutRef,
        shader_manager: &ShaderManagerMutRef,
        viewport: &ViewportMutRef,
        camera: &CameraMutRef,
        label: &'static str,
    ) -> BackgroundPass {
        let (attachments_descrs, attachments) = BackgroundPass::create_attachment_descrs(vk::Format::R8G8B8A8_SRGB);
        let subpass_def = BackgroundPass::create_subpass_def();

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        }];

        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments_descrs.as_ptr(),
            subpass_count: subpass_def.descriptions.len() as u32,
            p_subpasses: subpass_def.descriptions.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
            ..Default::default()
        };

        let render_pass = unsafe {
            device
                .borrow()
                .logical_device
                .create_render_pass(&render_pass_create_info, None)
                .expect("Could not create render pass")
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
        let viewport_ref = viewport.borrow();
        let pipeline = Pipeline::build(
            &device,
            shader_manager,
            render_pass,
            "background",
            viewport_ref.width,
            viewport_ref.height,
        )
        .with_layout_bindings(layout_bindings)
        .assamble();

        debug::Object::label(
            &device.borrow(),
            vk::ObjectType::RENDER_PASS,
            render_pass.as_raw(),
            label,
        );

        let drawable = FullScreenDrawable::new(&mut *resource_manager.borrow_mut());
        BackgroundPass {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            timer: Rc::clone(timer),
            viewport: Rc::clone(viewport),
            camera: Rc::clone(camera),
            pipeline,
            render_pass,
            label,
            drawable,
            attachments,
        }
    }

    fn create_attachment_descrs(format: vk::Format) -> (Vec<vk::AttachmentDescription>, Vec<Attachment>) {
        let attachment_descrs = vec![vk::AttachmentDescription {
            format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            final_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ..Default::default()
        }];

        let attachments = vec![
            Attachment::new(
                "Background",
                AttachmentSize::Relative(1.0, 1.0),
                format,
                AttachmentDirection::Write,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL
            ),
        ];

        (attachment_descrs, attachments)
    }

    fn create_subpass_def() -> SubpassDefinition {
        let attachment_refs = vec![vk::AttachmentReference {
            attachment: 0 as u32,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }];

        let descriptions = vec![vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: attachment_refs.len() as u32,
            p_color_attachments: attachment_refs.as_ptr(),
            ..Default::default()
        }];

        SubpassDefinition {
            attachment_refs,
            descriptions,
        }
    }
}

impl RenderPass for BackgroundPass {
    fn get_name(&self) -> &str {
        "Background"
    }

    fn run(&mut self, cmd_buffer: vk::CommandBuffer, attachments: Vec<vk::ImageView>) {
        let device = self.device.borrow();
        let mut _debug_region = debug::Region::new(&device, cmd_buffer, self.label);

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

        let viewport = self.viewport.borrow();
        let framebuffer = self.resource_manager.borrow_mut().framebuffer(viewport.width, viewport.height, attachments, self.render_pass);

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass,
            framebuffer: framebuffer.borrow().framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D { width: viewport.width, height: viewport.height },
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        unsafe {
            device.logical_device.cmd_begin_render_pass(
                cmd_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.logical_device.cmd_bind_pipeline(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipelines[0],
            );

            let mut resource_manager_ref = self.resource_manager.borrow_mut();
            self.drawable.draw(
                &device,
                &mut resource_manager_ref,
                &self.camera.borrow(),
                &self.timer.borrow(),
                cmd_buffer,
                &self.pipeline,
            );

            device.logical_device.cmd_end_render_pass(cmd_buffer);

        }
    }

    fn get_attachments(&self) -> &Vec<Attachment> {
        &self.attachments
    }
}

impl Drop for BackgroundPass {
    fn drop(&mut self) {
        unsafe {
            self.device
                .borrow()
                .logical_device
                .destroy_render_pass(self.render_pass, None);
        }
    }
}
