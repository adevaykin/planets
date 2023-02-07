use std::rc::Rc;
use crate::engine::camera::CameraMutRef;
use crate::engine::renderpass::RenderPass;
use crate::engine::viewport::{Viewport, ViewportMutRef};
use crate::vulkan::debug;
use crate::vulkan::device::{DeviceMutRef};
use crate::vulkan::drawable::{DrawType};
use crate::vulkan::pipeline::{Pipeline};
use crate::vulkan::resources::ResourceManagerMutRef;
use crate::vulkan::shader::{Binding, ShaderManager};
use ash::vk;
use ash::vk::{Handle};
use crate::engine::gameloop::GameLoopMutRef;
use crate::engine::scene::graph::SceneGraphMutRef;
use crate::vulkan::img::image::ImageMutRef;

pub const GEOMETRY_STENCIL_VAL: u32 = 1;

pub struct GBufferPass {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    gameloop: GameLoopMutRef,
    viewport: ViewportMutRef,
    camera: CameraMutRef,
    pipeline: Pipeline,
    pub render_pass: vk::RenderPass,
    scene: SceneGraphMutRef,
    color_attachment_imgs: Vec<ImageMutRef>,
    depth_attachment_img: ImageMutRef,
    attachment_descrs: Vec<(&'static str, vk::AttachmentDescription)>,
    depth_attachment_descr: (&'static str, vk::AttachmentDescription),
}

impl GBufferPass {
    pub fn new(
        device: &DeviceMutRef,
        resource_manager: &ResourceManagerMutRef,
        gameloop: &GameLoopMutRef,
        shader_manager: &mut ShaderManager,
        viewport: &ViewportMutRef,
        camera: &CameraMutRef,
        scene: &SceneGraphMutRef,
    ) -> Self {
        let attachments = GBufferPass::create_attachment_descrs();
        let mut attachment_descrs: Vec<vk::AttachmentDescription> =
            attachments.iter().map(|(_, descr)| *descr).collect();
        let mut attachment_refs = vec![];
        for (i, attachment) in attachment_descrs.iter().enumerate() {
            attachment_refs.push(vk::AttachmentReference {
                attachment: i as u32,
                layout: attachment.initial_layout,
            });
        }

        let depth_attachment = vk::AttachmentDescription {
            format: vk::Format::D32_SFLOAT_S8_UINT,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::CLEAR,
            stencil_store_op: vk::AttachmentStoreOp::STORE,
            initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };
        let depth_attachment_ref = [
            vk::AttachmentReference {
                attachment: attachment_refs.len() as u32,
                layout: depth_attachment.initial_layout
            }
        ];

        attachment_descrs.push(depth_attachment);

        let viewport_ref = viewport.borrow();
        let color_attachment_imgs = vec![
            resource_manager.borrow_mut().image(
                viewport_ref.width,
                viewport_ref.height,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC,
                "ColorAttachment",
            )
        ];

        let depth_attachment_img = resource_manager.borrow_mut().image(
            viewport_ref.width,
            viewport_ref.height,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            "DepthStencilAttachment",
        );

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        }];

        let subpass_descriptions = vec![vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: attachment_refs.len() as u32,
            p_color_attachments: attachment_refs.as_ptr(),
            p_depth_stencil_attachment: depth_attachment_ref.as_ptr(),
            ..Default::default()
        }];

        let render_pass_create_info = vk::RenderPassCreateInfo {
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

        let pipeline = GBufferPass::create_pipeline(device, shader_manager, &viewport.borrow(), render_pass);

        let pass = GBufferPass {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            gameloop: Rc::clone(gameloop),
            viewport: Rc::clone(viewport),
            camera: Rc::clone(camera),
            pipeline,
            render_pass,
            color_attachment_imgs,
            depth_attachment_img,
            attachment_descrs: attachments,
            depth_attachment_descr: ("DepthStencilAttachment", depth_attachment),
            scene: Rc::clone(scene),
        };

        debug::Object::label(
            &device.borrow(),
            vk::ObjectType::RENDER_PASS,
            render_pass.as_raw(),
            pass.get_name(),
        );

        pass
    }

    fn create_attachment_descrs() -> Vec<(&'static str, vk::AttachmentDescription)> {
        vec![
            (
                "GBuffer::Color",
                vk::AttachmentDescription {
                    format: vk::Format::R8G8B8A8_SRGB,
                    samples: vk::SampleCountFlags::TYPE_1,
                    load_op: vk::AttachmentLoadOp::CLEAR,
                    store_op: vk::AttachmentStoreOp::STORE,
                    initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    ..Default::default() }
           ),
        ]
    }

    fn create_pipeline(device: &DeviceMutRef, shader_manager: &mut ShaderManager, viewport: &Viewport, render_pass: vk::RenderPass) -> Pipeline {
        let layout_bindings = vec![
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Models as u32,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Lights as u32,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Timer as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Camera as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
        ];

        let front_stencil_op_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::REPLACE)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_mask(0)
            .write_mask(255)
            .compare_op(vk::CompareOp::ALWAYS)
            .reference(GEOMETRY_STENCIL_VAL);

        let back_stencil_op_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_mask(0)
            .write_mask(255)
            .compare_op(vk::CompareOp::NEVER)
            .reference(GEOMETRY_STENCIL_VAL);

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .stencil_test_enable(true)
            .front(*front_stencil_op_state)
            .back(*back_stencil_op_state)
            .depth_compare_op(vk::CompareOp::GREATER);

        Pipeline::build(
            device,
            shader_manager,
            render_pass,
            "gbuffer",
            viewport.width,
            viewport.height,
        )
            .with_layout_bindings(layout_bindings)
            .with_depth_stencil_info(*depth_stencil_info)
            .build()
    }
}

impl RenderPass for GBufferPass {
    fn get_name(&self) -> &str {
        "GBuffer"
    }

    fn run(&mut self, cmd_buffer: vk::CommandBuffer, _: Vec<ImageMutRef>) -> Vec<ImageMutRef> {
        let device = self.device.borrow();
        let mut _debug_region = debug::Region::new(&device, cmd_buffer, self.get_name());

        let mut attachment_views = vec![];

        {
            let mut attachment = self.color_attachment_imgs[0].borrow_mut();
            device.transition_layout(&mut attachment, self.attachment_descrs[0].1.initial_layout);
            match attachment.add_get_view(vk::Format::R8G8B8A8_SRGB) {
                Ok(view) => attachment_views.push(view),
                Err(msg) => log::error!("{}", msg),
            }

            let mut depth_attachment = self.depth_attachment_img.borrow_mut();
            device.transition_layout(&mut depth_attachment, self.depth_attachment_descr.1.initial_layout);
            match depth_attachment.add_get_view(vk::Format::D32_SFLOAT_S8_UINT) {
                Ok(view) => attachment_views.push(view),
                Err(msg) => log::error!("{}", msg),
            }
        }

        let viewport = self.viewport.borrow();
        // TODO: here and in Background pass: don't create new framebuffer on every run!
        // Framebuffers should be cached somehow
        let framebuffer = self.resource_manager.borrow_mut().framebuffer(
            viewport.width,
            viewport.height,
            &attachment_views,
            self.render_pass,
            "GBuffer"
        );

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass,
            framebuffer: framebuffer.borrow().framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: viewport.width,
                    height: viewport.height,
                },
            },
            clear_value_count: 2,
            p_clear_values: vec![
                vk::ClearValue::default(),
                vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: 0.0,
                        stencil: 10,
                    }
                }].as_ptr(),
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
                &[self.get_descriptor_set()],
                &[],
            );
        }

        self.scene.borrow().get_draw_list().borrow().write_draw_commands(DrawType::Opaque, &cmd_buffer);

        unsafe {
            device.logical_device.cmd_end_render_pass(cmd_buffer);
        }

        self.color_attachment_imgs[0].borrow_mut().set_layout(self.attachment_descrs[0].1.final_layout);
        self.depth_attachment_img.borrow_mut().set_layout(self.depth_attachment_descr.1.final_layout);

        vec![Rc::clone(&self.color_attachment_imgs[0]), Rc::clone(&self.depth_attachment_img)]
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
            range: gameloop.get_timer_ubo().buffer.borrow().size,
            ..Default::default()
        };

        let camera = self.camera.borrow();
        let camera_buffer_info = vk::DescriptorBufferInfo {
            buffer: camera.ubo.buffer.borrow().buffer,
            range: camera.ubo.buffer.borrow().size,
            ..Default::default()
        };

        let scene = self.scene.borrow();
        let model_data = scene.get_model_data();
        let models_buffer_info = vk::DescriptorBufferInfo {
            buffer: model_data.get_ssbo().borrow().buffer,
            range: model_data.get_ssbo().borrow().size,
            ..Default::default()
        };

        let lights = scene.get_light_manager().borrow();
        let lights_buffer_info = vk::DescriptorBufferInfo {
            buffer: lights.get_ssbo().borrow().buffer,
            range: lights.get_ssbo().borrow().size,
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
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: Binding::Models as u32,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &models_buffer_info,
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: Binding::Lights as u32,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &lights_buffer_info,
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

impl Drop for GBufferPass {
    fn drop(&mut self) {
        unsafe {
            self.device
                .borrow()
                .logical_device
                .destroy_render_pass(self.render_pass, None);
        }
    }
}
