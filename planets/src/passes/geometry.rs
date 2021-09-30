use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use crate::vulkan::renderpass::RenderPass;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::device::{DeviceMutRef};
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::shader::{ShaderManagerMutRef,Binding};
use crate::vulkan::framebuffer::Framebuffer;
use crate::vulkan::debug;
use crate::util::helpers::ViewportSize;
use crate::engine::camera::CameraMutRef;
use crate::vulkan::drawable::DrawType;
use crate::engine::scene::drawlist::DrawListMutRef;
use crate::engine::lights::LightManagerMutRef;

pub struct GeometryPass {
    device: DeviceMutRef,
    pipeline: Pipeline,
    pub render_pass: vk::RenderPass,
    label: String,
    framebuffers: Vec<Framebuffer>,
    light_manager: LightManagerMutRef,
    draw_list: DrawListMutRef,
    camera: CameraMutRef,
}

struct SubpassDefinition {
    #[allow(dead_code)]
    attachment_refs: Vec<vk::AttachmentReference>,
    #[allow(dead_code)]
    depth_attachment_ref: Vec<vk::AttachmentReference>,
    descriptions: Vec<vk::SubpassDescription>,
}

impl GeometryPass {
    pub fn new(device: &DeviceMutRef, swapchain: &Swapchain, shader_manager: &ShaderManagerMutRef,
        size_def: &impl ViewportSize, light_manager: &LightManagerMutRef, draw_list: &DrawListMutRef, camera: &CameraMutRef, label: &str) -> GeometryPass {
        let attachments = GeometryPass::create_attachments(swapchain.format, swapchain.depth_format);
        let subpass_def = GeometryPass::create_subpass_def();

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

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            ..Default::default()
        };

        let layout_bindings = vec![
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Lights as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: Binding::Camera as u32,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let pipeline = Pipeline::build(&device, shader_manager, render_pass, "basic", size_def.get_size().width as u32, size_def.get_size().height as u32)
            .with_depth_stencil_info(depth_stencil_info)
            .with_layout_bindings(layout_bindings)
            .assamble();

        let mut framebuffers = vec![];
        for i in 0..swapchain.images.len() {
            // TODO: care should be taken to access the correct view ASAP we will have more than one view for attachment
            debug_assert!(swapchain.images[i].views.len() == 1 && swapchain.depth_images[i].views.len() == 1);
            let attachment_views = vec![swapchain.images[i].views[0], swapchain.depth_images[i].views[0]];
            framebuffers.push(Framebuffer::new(device, swapchain, attachment_views, render_pass));
        }

        debug::Object::label(&device.borrow(), vk::ObjectType::RENDER_PASS, render_pass.as_raw(), label);

        GeometryPass {
            device: Rc::clone(device),
            pipeline,
            render_pass,
            label: String::from(label),
            framebuffers,
            light_manager: Rc::clone(light_manager),
            draw_list: Rc::clone(draw_list),
            camera: Rc::clone(camera) }
    }

    fn create_attachments(format: vk::Format, depth_format: vk::Format) -> Vec<vk::AttachmentDescription> {
        let attachment_descrs = vec![
            vk::AttachmentDescription {
                format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::LOAD,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: depth_format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
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

        let depth_attachment_ref = vec![vk::AttachmentReference {
                attachment: 1 as u32,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        }];

        let descriptions = vec![vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: attachment_refs.len() as u32,
            p_color_attachments: attachment_refs.as_ptr(),
            p_depth_stencil_attachment: depth_attachment_ref.as_ptr(),
            ..Default::default()
        }];

        SubpassDefinition { attachment_refs, depth_attachment_ref, descriptions }
    }

    fn record_commands(&self, swapchain: &Swapchain, frame_num: usize, command_buffer: &vk::CommandBuffer) {
        let mut _debug_region = debug::Region::new(&*self.device.borrow(), *command_buffer, self.label.as_str());

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
            {
                let device_ref = self.device.borrow_mut();
                device_ref.logical_device.cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                device_ref.logical_device.cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipelines[0]);
            }

            let light_manager_ref = self.light_manager.borrow();
            self.draw_list.borrow_mut().draw(frame_num, DrawType::Opaque, &self.camera.borrow(), &light_manager_ref,command_buffer, &self.pipeline);

            {
                let device_ref = self.device.borrow_mut();
                device_ref.logical_device.cmd_end_render_pass(*command_buffer);
            }
        }
    }
}

impl RenderPass for GeometryPass {
    fn draw_frame(&mut self, swapchain: &Swapchain, frame_num: usize, command_buffer: &vk::CommandBuffer) {
        self.record_commands(&swapchain, frame_num, command_buffer);
    }
}

impl Drop for GeometryPass {
    fn drop(&mut self) {
        unsafe {
            self.device.borrow().logical_device.destroy_render_pass(self.render_pass, None);
        }
    }
}
