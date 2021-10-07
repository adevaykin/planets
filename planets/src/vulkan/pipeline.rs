use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

use ash::vk;

use super::device::DeviceMutRef;
use super::shader::{Shader, ShaderManagerMutRef};
use crate::util::helpers::{SimpleViewportSize, ViewportSize};

pub struct Pipeline {
    device: DeviceMutRef,
    pub pipelines: Vec<vk::Pipeline>,
    pub layout: vk::PipelineLayout,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
}

struct ViewportDefinition {
    #[allow(dead_code)]
    /// TODO: use _ everywhere here instead of allow(dead_code)
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

impl Pipeline {
    pub fn build(
        device: &DeviceMutRef,
        shader_manager: &ShaderManagerMutRef,
        render_pass: vk::RenderPass,
        shader_name: &str,
        width: u32,
        height: u32,
    ) -> PipelineAssembly {
        PipelineAssembly {
            device: Rc::clone(device),
            shader_manager: Rc::clone(shader_manager),
            render_pass,
            shader_name: String::from(shader_name),
            viewport_size: SimpleViewportSize::from_width_height(width, height),
            layout_bindings: vec![],
            depth_stencil_info: None,
            vertex_input_binding_description: None,
            vertex_input_attribute_descriptions: None,
        }
    }

    fn create_fragment_stage_info(shader: &Shader) -> ShaderStageDefinition {
        let main_func_name = CString::new("main").unwrap();
        let create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            p_specialization_info: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: shader.fragment_module.unwrap(),
            p_name: main_func_name.as_ptr(),
        };

        ShaderStageDefinition {
            main_func_name,
            create_info,
        }
    }

    fn create_vertex_stage_info(shader: &Shader) -> ShaderStageDefinition {
        let main_func_name = CString::new("main").unwrap();
        let create_info = vk::PipelineShaderStageCreateInfo {
            s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next: ptr::null(),
            p_specialization_info: ptr::null(),
            flags: vk::PipelineShaderStageCreateFlags::empty(),
            stage: vk::ShaderStageFlags::VERTEX,
            module: shader.vertex_module.unwrap(),
            p_name: main_func_name.as_ptr(),
        };

        ShaderStageDefinition {
            main_func_name,
            create_info,
        }
    }

    fn create_input_assembly_state_info() -> vk::PipelineInputAssemblyStateCreateInfo {
        vk::PipelineInputAssemblyStateCreateInfo {
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        }
    }

    fn create_viewport_state_def(size_def: &impl ViewportSize) -> ViewportDefinition {
        let size = size_def.get_size();
        let viewports = vec![vk::Viewport {
            x: size.offset_x,
            y: size.offset_y,
            width: size.width,
            height: size.height,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = vec![vk::Rect2D {
            offset: vk::Offset2D {
                x: size.offset_x as i32,
                y: size.offset_y as i32,
            },
            extent: vk::Extent2D {
                width: size.width as u32,
                height: size.height as u32,
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
            color_write_mask: vk::ColorComponentFlags::all(),
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

        let layout = unsafe {
            device
                .create_pipeline_layout(&create_info, None)
                .expect("Failed to create pipeline layout")
        };

        layout
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

pub struct PipelineAssembly {
    device: DeviceMutRef,
    shader_manager: ShaderManagerMutRef,
    render_pass: vk::RenderPass,
    shader_name: String,
    viewport_size: SimpleViewportSize,

    layout_bindings: Vec<vk::DescriptorSetLayoutBinding>,
    depth_stencil_info: Option<vk::PipelineDepthStencilStateCreateInfo>,
    vertex_input_binding_description: Option<vk::VertexInputBindingDescription>,
    vertex_input_attribute_descriptions: Option<Vec<vk::VertexInputAttributeDescription>>,
}

impl PipelineAssembly {
    pub fn assamble(&mut self) -> Pipeline {
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
    ) -> &mut PipelineAssembly {
        self.depth_stencil_info = Some(info);
        self
    }

    pub fn with_layout_bindings(
        &mut self,
        layout_bindings: Vec<vk::DescriptorSetLayoutBinding>,
    ) -> &mut PipelineAssembly {
        self.layout_bindings = layout_bindings;
        self
    }

    pub fn with_vertex_input_binding_description(
        &mut self,
        bindings: vk::VertexInputBindingDescription,
    ) -> &mut PipelineAssembly {
        self.vertex_input_binding_description = Some(bindings);
        self
    }

    pub fn with_vertex_input_attribute_description(
        &mut self,
        attributes: Vec<vk::VertexInputAttributeDescription>,
    ) -> &mut PipelineAssembly {
        self.vertex_input_attribute_descriptions = Some(attributes);
        self
    }

    fn create_graphics_pipelines(
        &mut self,
    ) -> (
        vk::PipelineLayout,
        vk::DescriptorSetLayout,
        Vec<vk::Pipeline>,
    ) {
        let mut shader_manager = self.shader_manager.borrow_mut();
        let shader = shader_manager.get_shader(self.shader_name.as_str());

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
        let viewport_state = Pipeline::create_viewport_state_def(&self.viewport_size);
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
            layout: layout,
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