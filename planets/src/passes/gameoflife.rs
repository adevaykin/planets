use std::rc::Rc;

use crate::engine::camera::CameraMutRef;
use crate::engine::framegraph::RenderPass;
use crate::engine::timer::TimerMutRef;
use crate::engine::viewport::ViewportMutRef;
use crate::vulkan::debug;
use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::drawable::FullScreenDrawable;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::resources::ResourceManagerMutRef;
use crate::vulkan::shader::{Binding, ShaderManagerMutRef};
use ash::vk;
use ash::vk::{CommandBuffer, Handle, ImageView};

pub struct GameOfLifePass {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    timer: TimerMutRef,
    viewport: ViewportMutRef,
    camera: CameraMutRef,
    pipeline: Pipeline,
    pub render_pass: vk::RenderPass,
    drawable: FullScreenDrawable,
    attachments: Vec<(&'static str, vk::AttachmentDescription)>,
}

impl GameOfLifePass {
    pub fn new(
        device: &DeviceMutRef,
        resource_manager: &ResourceManagerMutRef,
        timer: &TimerMutRef,
        shader_manager: &ShaderManagerMutRef,
        viewport: &ViewportMutRef,
        camera: &CameraMutRef,
    ) -> Self {
        let attachments = GameOfLifePass::create_attachment_descrs(vk::Format::R8G8B8A8_SRGB);

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        }];

        let attachment_descrs: Vec<vk::AttachmentDescription> =
            attachments.iter().map(|(_, descr)| *descr).collect();
        let mut attachment_refs = vec![];
        for (i, attachment) in attachment_descrs.iter().enumerate() {
            attachment_refs.push(vk::AttachmentReference {
                attachment: i as u32,
                layout: attachment.initial_layout,
            });
        }

        let subpass_descriptions = vec![vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: attachment_refs.len() as u32,
            p_color_attachments: attachment_refs.as_ptr(),
            ..Default::default()
        }];

        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            attachment_count: attachment_descrs.len() as u32,
            p_attachments: attachment_descrs.as_ptr(),
            subpass_count: subpass_descriptions.len() as u32,
            p_subpasses: subpass_descriptions.as_ptr(),
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
            "gameoflife",
            viewport_ref.width,
            viewport_ref.height,
        )
        .with_layout_bindings(layout_bindings)
        .assamble();

        let drawable = FullScreenDrawable::new(&mut *resource_manager.borrow_mut());

        let pass = GameOfLifePass {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            timer: Rc::clone(timer),
            viewport: Rc::clone(viewport),
            camera: Rc::clone(camera),
            pipeline,
            render_pass,
            drawable,
            attachments,
        };

        debug::Object::label(
            &device.borrow(),
            vk::ObjectType::RENDER_PASS,
            render_pass.as_raw(),
            pass.get_name(),
        );

        pass
    }

    fn create_attachment_descrs(
        format: vk::Format,
    ) -> Vec<(&'static str, vk::AttachmentDescription)> {
        let attachments = vec![(
            "Background",
            vk::AttachmentDescription {
                format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::LOAD,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                ..Default::default()
            },
        )];

        attachments
    }
}

impl RenderPass for GameOfLifePass {
    fn get_name(&self) -> &str {
        "GameOfLife"
    }

    fn run(&mut self, cmd_buffer: CommandBuffer, attachments: &Vec<ImageView>) {
        let device = self.device.borrow();
        let mut _debug_region = debug::Region::new(&device, cmd_buffer, self.get_name());

        let viewport = self.viewport.borrow();
        // TODO: here and in Background pass: don't create new framebuffer on every run!
        // Framebuffers should be cached somehow
        let framebuffer = self.resource_manager.borrow_mut().framebuffer(
            viewport.width,
            viewport.height,
            attachments,
            self.render_pass,
        );

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass,
            framebuffer: framebuffer.borrow().framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: viewport.width,
                    height: viewport.height,
                },
            },
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

    fn get_attachments(&self) -> &Vec<(&'static str, vk::AttachmentDescription)> {
        &self.attachments
    }
}
