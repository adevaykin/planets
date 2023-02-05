use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

use ash::vk;

use super::device::DeviceMutRef;
use super::shader::{Shader};
use crate::vulkan::shader::ShaderManager;

pub struct Pipeline {
    device: DeviceMutRef,
    pub pipelines: Vec<vk::Pipeline>,
    pub layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
}

struct ViewportDefinition {
    #[allow(dead_code)]
    viewports: Vec<vk::Viewport>,
    #[allow(dead_code)]
    scissors: Vec<vk::Rect2D>,
    create_info: vk::PipelineViewportStateCreateInfo,
}

struct ColorAttachmentDef {
    #[allow(dead_code)]
    attachments: Vec<vk::PipelineColorBlendAttachmentState>,
    create_info: vk::PipelineColorBlendStateCreateInfo,
}

struct ShaderStageDefinition {
    #[allow(dead_code)]
    main_func_name: CString,
    create_info: vk::PipelineShaderStageCreateInfo,
}

impl<'a> Pipeline {
    pub fn build(device: &DeviceMutRef, shader_manager: &'a mut ShaderManager, render_pass: vk::RenderPass,
                 shader_name: &str, width: u32, height: u32) -> PipelineBuilder<'a>
    {
        PipelineBuilder::new(device, shader_manager, render_pass, shader_name, width, height)
    }

    fn create_fragment_stage_info(shader: &Shader) -> ShaderStageDefinition {
        let main_func_name = CString::new("main").unwrap();
        let create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: shader.fragment_module.unwrap(),
            p_name: main_func_name.as_ptr(),
            ..Default::default()
        };

        ShaderStageDefinition {
            main_func_name,
            create_info,
        }
    }

    fn create_vertex_stage_info(shader: &Shader) -> ShaderStageDefinition {
        let main_func_name = CString::new("main").unwrap();
        let create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: shader.vertex_module.unwrap(),
            p_name: main_func_name.as_ptr(),
            ..Default::default()
        };

        ShaderStageDefinition {
            main_func_name,
            create_info,
        }
    }

    fn create_input_assembly_state_info() -> vk::PipelineInputAssemblyStateCreateInfo {
        vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        }
    }

    fn create_viewport_state_def(width: u32, height: u32, offset_x: u32, offset_y: u32) -> ViewportDefinition {
        let viewports = vec![vk::Viewport {
            x: offset_x as f32,
            y: offset_y as f32,
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = vec![vk::Rect2D {
            offset: vk::Offset2D {
                x: offset_x as i32,
                y: offset_y as i32,
            },
            extent: vk::Extent2D {
                width,
                height,
            },
        }];

        let create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
        };

        ViewportDefinition {
            viewports,
            scissors,
            create_info,
        }
    }

    fn create_rasterization_state_info() -> vk::PipelineRasterizationStateCreateInfo {
        vk::PipelineRasterizationStateCreateInfo {
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            ..Default::default()
        }
    }

    fn create_multisample_state_info() -> vk::PipelineMultisampleStateCreateInfo {
        vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        }
    }

    fn create_color_blend_attachment_state_def() -> ColorAttachmentDef {
        let attachments = vec![vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            blend_enable: vk::FALSE,
            ..Default::default()
        }];

        let create_info = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            logic_op_enable: vk::FALSE,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            ..Default::default()
        };

        ColorAttachmentDef {
            attachments,
            create_info,
        }
    }

    fn create_descriptor_set_layout(
        device: &DeviceMutRef,
        layout_bindings: &Vec<vk::DescriptorSetLayoutBinding>,
    ) -> vk::DescriptorSetLayout {
        let create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            binding_count: layout_bindings.len() as u32,
            p_bindings: layout_bindings.as_ptr(),
            ..Default::default()
        };

        let layout = unsafe {
            device
                .borrow()
                .logical_device
                .create_descriptor_set_layout(&create_info, None)
                .expect("Failed to create descriptor set layout")
        };

        layout
    }

    fn create_layout(
        device: &ash::Device,
        descriptor_set_layout: &vk::DescriptorSetLayout,
    ) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            set_layout_count: 1,
            p_set_layouts: descriptor_set_layout,
            ..Default::default()
        };

        unsafe {
            device
                .create_pipeline_layout(&create_info, None)
                .expect("Failed to create pipeline layout")
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device
                .borrow()
                .logical_device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.device
                .borrow()
                .logical_device
                .destroy_pipeline_layout(self.layout, None);
            for pipeline in &self.pipelines {
                self.device
                    .borrow()
                    .logical_device
                    .destroy_pipeline(*pipeline, None);
            }
        }
    }
}

pub struct PipelineBuilder<'a> {
    device: DeviceMutRef,
    shader_manager: &'a mut ShaderManager,
    render_pass: vk::RenderPass,
    shader_name: String,
    viewport_width: u32,
    viewport_height: u32,

    layout_bindings: Vec<vk::DescriptorSetLayoutBinding>,
    depth_stencil_info: Option<vk::PipelineDepthStencilStateCreateInfo>,
    vertex_input_binding_description: Option<vk::VertexInputBindingDescription>,
    vertex_input_attribute_descriptions: Option<Vec<vk::VertexInputAttributeDescription>>,
    dynamic_state: Option<vk::PipelineDynamicStateCreateInfo>,
}

impl<'a> PipelineBuilder<'a> {
    fn new(device: &DeviceMutRef, shader_manager: &'a mut ShaderManager, render_pass: vk::RenderPass,
               shader_name: &str, width: u32, height: u32
    ) -> Self {
        PipelineBuilder {
            device: Rc::clone(device),
            shader_manager,
            render_pass,
            shader_name: String::from(shader_name),
            viewport_width: width,
            viewport_height: height,
            layout_bindings: vec![],
            depth_stencil_info: None,
            vertex_input_binding_description: None,
            vertex_input_attribute_descriptions: None,
            dynamic_state: None,
        }
    }

    pub fn build(&mut self) -> Pipeline {
        let (layout, descriptor_set_layout, pipelines) = self.create_graphics_pipelines();
        Pipeline {
            device: Rc::clone(&self.device),
            pipelines,
            layout,
            descriptor_set_layout,
        }
    }

    pub fn with_depth_stencil_info(
        &mut self,
        info: vk::PipelineDepthStencilStateCreateInfo,
    ) -> &mut Self {
        self.depth_stencil_info = Some(info);
        self
    }

    #[allow(dead_code)]
    pub fn with_dynamic_state(&mut self, state: vk::PipelineDynamicStateCreateInfo) -> &mut Self {
        self.dynamic_state = Some(state);
        self
    }

    pub fn with_layout_bindings(
        &mut self,
        layout_bindings: Vec<vk::DescriptorSetLayoutBinding>,
    ) ->&mut Self {
        self.layout_bindings = layout_bindings;
        self
    }

    #[allow(dead_code)]
    pub fn with_vertex_input_binding_description(
        &mut self,
        bindings: vk::VertexInputBindingDescription,
    ) -> &mut Self {
        self.vertex_input_binding_description = Some(bindings);
        self
    }

    #[allow(dead_code)]
    pub fn with_vertex_input_attribute_description(
        &mut self,
        attributes: Vec<vk::VertexInputAttributeDescription>,
    ) {
        self.vertex_input_attribute_descriptions = Some(attributes);
    }

    fn create_graphics_pipelines(
        &mut self,
    ) -> (
        vk::PipelineLayout,
        vk::DescriptorSetLayout,
        Vec<vk::Pipeline>,
    ) {
        let shader = self.shader_manager.get_shader(self.shader_name.as_str());

        let stages_def = vec![
            Pipeline::create_vertex_stage_info(shader),
            Pipeline::create_fragment_stage_info(shader),
        ];

        let stages = vec![stages_def[0].create_info, stages_def[1].create_info];

        let binding_desc = match self.vertex_input_binding_description {
            Some(description) => description,
            None => super::drawable::get_default_vertex_input_binding_description(),
        };

        let attribute_descs = match self.vertex_input_attribute_descriptions.take() {
            Some(descriptions) => descriptions,
            None => super::drawable::get_default_attribute_descriptions(),
        };

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &binding_desc,
            vertex_attribute_description_count: attribute_descs.len() as u32,
            p_vertex_attribute_descriptions: attribute_descs.as_ptr(),
        };

        let input_assembly_state = Pipeline::create_input_assembly_state_info();
        let viewport_state = Pipeline::create_viewport_state_def(self.viewport_width, self.viewport_height, 0, 0);
        let rasterization_state = Pipeline::create_rasterization_state_info();
        let depth_stencil_state = match self.depth_stencil_info {
            None => vk::PipelineDepthStencilStateCreateInfo {
                ..Default::default()
            },
            Some(info) => info,
        };
        let multisample_state = Pipeline::create_multisample_state_info();
        let color_blend_state = Pipeline::create_color_blend_attachment_state_def();
        let descriptor_set_layout =
            Pipeline::create_descriptor_set_layout(&self.device, &self.layout_bindings);
        let layout =
            Pipeline::create_layout(&self.device.borrow().logical_device, &descriptor_set_layout);
        let dynamic_state = if let Some(dynamic_state) = self.dynamic_state { &dynamic_state } else { ptr::null() };

        let create_info = vec![vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            stage_count: stages.len() as u32,
            p_stages: stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state,
            p_input_assembly_state: &input_assembly_state,
            p_viewport_state: &viewport_state.create_info,
            p_rasterization_state: &rasterization_state,
            p_depth_stencil_state: &depth_stencil_state,
            p_multisample_state: &multisample_state,
            p_color_blend_state: &color_blend_state.create_info,
            p_dynamic_state: dynamic_state,
            layout,
            render_pass: self.render_pass,
            subpass: 0,
            ..Default::default()
        }];

        let pipeline = unsafe {
            self.device
                .borrow()
                .logical_device
                .create_graphics_pipelines(vk::PipelineCache::null(), &create_info, None)
                .expect("Failed to create graphics pipeline")
        };

        (layout, descriptor_set_layout, pipeline)
    }
}
