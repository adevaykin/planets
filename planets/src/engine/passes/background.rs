use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use crate::engine::viewport::{ViewportMutRef};
use crate::vulkan::debug;
use crate::vulkan::device::{DeviceMutRef};
use crate::vulkan::drawable::FullScreenDrawable;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::resources::manager::{ResourceManagerMutRef};
use crate::vulkan::shader::{Binding, ShaderManager};

use crate::engine::camera::CameraMutRef;
use crate::engine::renderpass::{RenderPass};
use crate::engine::gameloop::GameLoopMutRef;
use crate::engine::passes::gbuffer::GEOMETRY_STENCIL_VAL;
use crate::vulkan::debug::DebugResource;
use crate::vulkan::img::image::{ImageMutRef};

pub struct BackgroundPass {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    gameloop: GameLoopMutRef,
    viewport: ViewportMutRef,
    camera: CameraMutRef,
    pipeline: Pipeline,
    pub render_pass: vk::RenderPass,
    drawable: FullScreenDrawable,
    attachment_descrs: Vec<(&'static str, vk::AttachmentDescription)>,
    depth_attachment_descr: (&'static str, vk::AttachmentDescription),
    label: String,
}

impl BackgroundPass {
    pub fn new(
        device: &DeviceMutRef,
        resource_manager: &ResourceManagerMutRef,
        gameloop: &GameLoopMutRef,
        shader_manager: &mut ShaderManager,
        viewport: &ViewportMutRef,
        camera: &CameraMutRef,
    ) -> BackgroundPass {
        let attachments = BackgroundPass::create_attachment_descrs(vk::Format::R8G8B8A8_SRGB);

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
            load_op: vk::AttachmentLoadOp::LOAD,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::LOAD,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
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

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
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

        let front_stencil_op_state = vk::StencilOpState::builder()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .depth_fail_op(vk::StencilOp::KEEP)
            .compare_mask(255)
            .write_mask(0)
            .compare_op(vk::CompareOp::NOT_EQUAL)
            .reference(GEOMETRY_STENCIL_VAL);

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .stencil_test_enable(true)
            .front(*front_stencil_op_state);

        let viewport_ref = viewport.borrow();
        let pipeline = Pipeline::build(
            device,
            shader_manager,
            render_pass,
            "background",
            viewport_ref.width,
            viewport_ref.height,
        )
        .with_layout_bindings(layout_bindings)
        .with_depth_stencil_info(*depth_stencil_info)
        .build();

        let drawable = FullScreenDrawable::new(&mut resource_manager.borrow_mut());
        let pass = BackgroundPass {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            gameloop: Rc::clone(gameloop),
            viewport: Rc::clone(viewport),
            camera: Rc::clone(camera),
            pipeline,
            render_pass,
            drawable,
            attachment_descrs: attachments,
            depth_attachment_descr: ("DepthStencilAttachment", depth_attachment),
            label: String::from("Background"),
        };
        debug::Object::label(&device.borrow(), &pass);

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

impl DebugResource for BackgroundPass {
    fn get_type(&self) -> vk::ObjectType {
        vk::ObjectType::RENDER_PASS
    }

    fn get_handle(&self) -> u64 {
        self.render_pass.as_raw()
    }

    fn get_label(&self) -> &String {
        &self.label
    }
}

impl RenderPass for BackgroundPass {
    fn run(&mut self, cmd_buffer: vk::CommandBuffer, input_attachments: Vec<ImageMutRef>) -> Vec<ImageMutRef> {
        let device = self.device.borrow();
        let mut _debug_region = debug::Region::new(&device, self.get_label().as_str());

        let mut attachment_views = vec![];
        {
            let mut attachment = input_attachments[0].borrow_mut();
            device.transition_layout(&mut attachment, self.attachment_descrs[0].1.initial_layout);
            match attachment.add_get_view(vk::Format::R8G8B8A8_SRGB) {
                Ok(view) => attachment_views.push(view),
                Err(msg) => log::error!("{}", msg)
            }

            let mut depth_attachment = input_attachments[1].borrow_mut();
            device.transition_layout(&mut depth_attachment, self.depth_attachment_descr.1.initial_layout);
            match depth_attachment.add_get_view(vk::Format::D32_SFLOAT_S8_UINT) {
                Ok(view) => attachment_views.push(view),
                Err(msg) => log::error!("{}", msg),
            }
        }

        let viewport = self.viewport.borrow();
        let framebuffer = self.resource_manager.borrow_mut().framebuffer(
            viewport.width,
            viewport.height,
            &attachment_views,
            self.render_pass,
            "Background"
        );

        match self.get_descriptor_set()  {
            Ok(descriptor_set) => {
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

                input_attachments[0].borrow_mut().set_layout(self.attachment_descrs[0].1.final_layout);
                input_attachments[1].borrow_mut().set_layout(self.depth_attachment_descr.1.final_layout);
            },
            Err(msg) => {
                log::error!("Failed to execute Background render pass: {}", msg);
            }
        };

        input_attachments
    }

    fn get_pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    fn get_descriptor_set(&self) -> Result<vk::DescriptorSet,&'static str> {
        match self
            .resource_manager
            .borrow_mut()
            .descriptor_set_manager
            .allocate_descriptor_set(&self.pipeline.descriptor_set_layout) {
            Ok(descriptor_set) => {
                let device_ref = self.device.borrow();
                let timer_buffer_info = self.gameloop.borrow().get_descriptor_buffer_info(device_ref.get_image_idx());
                let camera_buffer_info = self.camera.borrow().get_descriptor_buffer_info(device_ref.get_image_idx());

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
                    device_ref
                        .logical_device
                        .update_descriptor_sets(&descr_set_writes, &[]);
                }

                Ok(descriptor_set)
            },
            Err(msg) => Err(msg)
        }
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
