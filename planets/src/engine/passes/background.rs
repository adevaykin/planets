use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use crate::engine::viewport::ViewportMutRef;
use crate::vulkan::debug;
use crate::vulkan::device::{DeviceMutRef};
use crate::vulkan::drawable::FullScreenDrawable;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::resources::{ResourceManagerMutRef};
use crate::vulkan::shader::{Binding, ShaderManagerMutRef};

use crate::engine::camera::CameraMutRef;
use crate::engine::renderpass::{RenderPass};
use crate::engine::gameloop::GameLoopMutRef;
use crate::vulkan::image::image::{ImageMutRef};

pub struct BackgroundPass {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    gameloop: GameLoopMutRef,
    viewport: ViewportMutRef,
    camera: CameraMutRef,
    pipeline: Pipeline,
    pub render_pass: vk::RenderPass,
    drawable: FullScreenDrawable,
    attachments: Vec<(&'static str, vk::AttachmentDescription)>,
}

impl BackgroundPass {
    pub fn new(
        device: &DeviceMutRef,
        resource_manager: &ResourceManagerMutRef,
        gameloop: &GameLoopMutRef,
        shader_manager: &ShaderManagerMutRef,
        viewport: &ViewportMutRef,
        camera: &CameraMutRef,
    ) -> BackgroundPass {
        let attachments = BackgroundPass::create_attachment_descrs(vk::Format::R8G8B8A8_SRGB);

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
            "background",
            viewport_ref.width,
            viewport_ref.height,
        )
        .with_layout_bindings(layout_bindings)
        .assamble();

        let drawable = FullScreenDrawable::new(&mut *resource_manager.borrow_mut());
        let pass = BackgroundPass {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            gameloop: Rc::clone(gameloop),
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
                load_op: vk::AttachmentLoadOp::DONT_CARE,
                store_op: vk::AttachmentStoreOp::STORE,
                initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
        )];

        attachments
    }
}

impl RenderPass for BackgroundPass {
    fn get_name(&self) -> &str {
        "Background"
    }

    fn run(&mut self, cmd_buffer: vk::CommandBuffer) -> Vec<ImageMutRef> {
        let device = self.device.borrow();
        let mut _debug_region = debug::Region::new(&device, cmd_buffer, self.get_name());

        let attachment_views = vec![];

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
        let framebuffer = self.resource_manager.borrow_mut().framebuffer(
            viewport.width,
            viewport.height,
            &attachment_views,
            self.render_pass,
        );

        let descriptor_set = self.get_descriptor_set();

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
            device.logical_device.cmd_bind_descriptor_sets(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[descriptor_set],
                &[],
            );

            self.drawable.draw(
                &device,
                cmd_buffer,
            );

            device.logical_device.cmd_end_render_pass(cmd_buffer);
        }

        vec![]
    }

    fn get_pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    fn get_descriptor_set(&self) -> vk::DescriptorSet {
        let descriptor_set = self
            .resource_manager
            .borrow_mut()
            .descriptor_set_manager
            .allocate_descriptor_set(&self.device.borrow(), &self.pipeline.descriptor_set_layout);

        let gameloop = self.gameloop.borrow();
        let timer_buffer_info = vk::DescriptorBufferInfo {
            buffer: gameloop.get_timer_ubo().buffer.borrow().buffer,
            range: gameloop.get_timer_ubo().buffer.borrow().size as u64,
            ..Default::default()
        };

        let camera = self.camera.borrow();
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
            self.device
                .borrow()
                .logical_device
                .update_descriptor_sets(&descr_set_writes, &[]);
        }

        descriptor_set
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
