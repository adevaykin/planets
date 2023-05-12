use alloc::rc::Rc;
use ash::vk;
use crate::engine::renderpass::RenderPass;
use crate::engine::scene::graph::SceneGraphMutRef;
use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::img::image::{ImageAccess, ImageMutRef};
use crate::vulkan::mem::{AllocatedBufferMutRef, BufferAccess, VecBufferData};
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::resources::manager::{AttachmentSize, ResourceManager, ResourceManagerMutRef};
use crate::vulkan::rt::r#as::AccelerationStructure;
use crate::vulkan::shader::ShaderManager;

pub struct RaytracedAo {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    scene: SceneGraphMutRef,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    shader_binding_table_buffer: AllocatedBufferMutRef,
    image: ImageMutRef,
    color_buffer: AllocatedBufferMutRef,
    accel: Option<AccelerationStructure>,
}

impl RaytracedAo {
    #[cfg(not(target_os = "macos"))]
    pub fn new(device: &DeviceMutRef,
               resource_manager: &ResourceManagerMutRef,
               shader_manager: &mut ShaderManager, scene: &SceneGraphMutRef) -> Option<Self> {

        let image = resource_manager.borrow_mut().attachment(AttachmentSize::Fixed(512, 512), vk::Format::R8G8B8A8_SNORM, vk::ImageUsageFlags::STORAGE, "RtImage");
        let color= vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0];
        let color_data = VecBufferData::new(&color);
        let color_buffer = resource_manager.borrow_mut().buffer_with_staging(&color_data, vk::BufferUsageFlags::STORAGE_BUFFER, "RtColorBuffer");

        let (pipeline, pipeline_layout, descriptor_set_layout, shader_binding_table_buffer) = Self::create_pipeline(device, shader_manager, &mut resource_manager.borrow_mut());

        Some(RaytracedAo {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            scene: Rc::clone(scene),
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            shader_binding_table_buffer,
            image,
            color_buffer,
            accel: None,
        })
    }

    #[cfg(target_os = "macos")]
    pub fn new(_: &DeviceMutRef,
               _: &ResourceManagerMutRef,
               _: &mut ShaderManager,) -> Option<Self> {
        None
    }

    fn create_pipeline(device: &DeviceMutRef, shader_manager: &mut ShaderManager, resource_manager: &mut ResourceManager) -> (vk::Pipeline, vk::PipelineLayout, vk::DescriptorSetLayout, AllocatedBufferMutRef) {
        let device_ref = device.borrow();

        let (descriptor_set_layout, graphics_pipeline, pipeline_layout, shader_group_count) = {
            let binding_flags_inner = [
                vk::DescriptorBindingFlagsEXT::empty(),
                vk::DescriptorBindingFlagsEXT::empty(),
                vk::DescriptorBindingFlagsEXT::empty(),
            ];

            let mut binding_flags = vk::DescriptorSetLayoutBindingFlagsCreateInfoEXT::builder()
                .binding_flags(&binding_flags_inner)
                .build();

            let descriptor_set_layout = unsafe {
                device_ref.logical_device.create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::builder()
                        .bindings(&[
                            vk::DescriptorSetLayoutBinding::builder()
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                                .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                                .binding(0)
                                .build(),
                            vk::DescriptorSetLayoutBinding::builder()
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                                .stage_flags(vk::ShaderStageFlags::RAYGEN_KHR)
                                .binding(1)
                                .build(),
                            vk::DescriptorSetLayoutBinding::builder()
                                .descriptor_count(1)
                                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                                .stage_flags(vk::ShaderStageFlags::CLOSEST_HIT_KHR)
                                .binding(2)
                                .build(),
                        ])
                        .push_next(&mut binding_flags)
                        .build(),
                    None,
                )
            }
                .unwrap();

            let shader = shader_manager.get_shader("rtao");

            let layouts = vec![descriptor_set_layout];
            let layout_create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&layouts);

            let pipeline_layout =
                unsafe {
                    device_ref.logical_device.create_pipeline_layout(&layout_create_info, None)
                }.unwrap();

            let shader_groups = vec![
                // group0 = [ raygen ]
                vk::RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(0)
                    .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                    .any_hit_shader(vk::SHADER_UNUSED_KHR)
                    .intersection_shader(vk::SHADER_UNUSED_KHR)
                    .build(),
                // group1 = [ chit ]
                vk::RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(vk::RayTracingShaderGroupTypeKHR::TRIANGLES_HIT_GROUP)
                    .general_shader(vk::SHADER_UNUSED_KHR)
                    .closest_hit_shader(1)
                    .any_hit_shader(vk::SHADER_UNUSED_KHR)
                    .intersection_shader(vk::SHADER_UNUSED_KHR)
                    .build(),
                // group2 = [ miss ]
                vk::RayTracingShaderGroupCreateInfoKHR::builder()
                    .ty(vk::RayTracingShaderGroupTypeKHR::GENERAL)
                    .general_shader(2)
                    .closest_hit_shader(vk::SHADER_UNUSED_KHR)
                    .any_hit_shader(vk::SHADER_UNUSED_KHR)
                    .intersection_shader(vk::SHADER_UNUSED_KHR)
                    .build(),
            ];

            let shader_stages = vec![
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::RAYGEN_KHR)
                    .module(shader.raygen_module.as_ref().unwrap().get_module())
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
                    .build(),
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::CLOSEST_HIT_KHR)
                    .module(shader.chit_module.as_ref().unwrap().get_module())
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
                    .build(),
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(vk::ShaderStageFlags::MISS_KHR)
                    .module(shader.miss_module.as_ref().unwrap().get_module())
                    .name(std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap())
                    .build(),
            ];

            let pipeline = unsafe {
                device.borrow().rt_pipeline.pipeline.create_ray_tracing_pipelines(
                    vk::DeferredOperationKHR::null(),
                    vk::PipelineCache::null(),
                    &[vk::RayTracingPipelineCreateInfoKHR::builder()
                        .stages(&shader_stages)
                        .groups(&shader_groups)
                        .max_pipeline_ray_recursion_depth(1)
                        .layout(pipeline_layout)
                        .build()],
                    None,
                )
            }
            .unwrap()[0];

            (
                descriptor_set_layout,
                pipeline,
                pipeline_layout,
                shader_groups.len(),
            )
        };

        let rt_pipeline_properties = &device.borrow().rt_pipeline.properties;

        let handle_size_aligned = Self::aligned_size(
            rt_pipeline_properties.shader_group_handle_size,
            rt_pipeline_properties.shader_group_base_alignment,
        ) as u64;

        let shader_binding_table_buffer = {
            let incoming_table_data = unsafe {
                device.borrow().rt_pipeline.pipeline.get_ray_tracing_shader_group_handles(
                    graphics_pipeline,
                    0,
                    shader_group_count as u32,
                    shader_group_count * rt_pipeline_properties.shader_group_handle_size as usize,
                )
            }
                .unwrap();

            let table_size = shader_group_count * handle_size_aligned as usize;
            let mut table_data = vec![0u8; table_size];

            for i in 0..shader_group_count {
                table_data[i * handle_size_aligned as usize
                    ..i * handle_size_aligned as usize
                    + rt_pipeline_properties.shader_group_handle_size as usize]
                    .copy_from_slice(
                        &incoming_table_data[i * rt_pipeline_properties.shader_group_handle_size
                            as usize
                            ..i * rt_pipeline_properties.shader_group_handle_size as usize
                            + rt_pipeline_properties.shader_group_handle_size as usize],
                    );
            }

            let buffer_data = VecBufferData::new(&table_data);
            let shader_binding_table_buffer = resource_manager.buffer_with_staging(
                &buffer_data,
                vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_BINDING_TABLE_KHR,
                "ShaderBindingTableBuffer",
            );

            shader_binding_table_buffer
        };

        (graphics_pipeline, pipeline_layout, descriptor_set_layout, shader_binding_table_buffer)
    }

    fn aligned_size(value: u32, alignment: u32) -> u32 {
        (value + alignment - 1) & !(alignment - 1)
    }
}

impl RenderPass for RaytracedAo {
    fn run(&mut self, cmd_buffer: vk::CommandBuffer, _: Vec<ImageMutRef>) -> Vec<ImageMutRef> {
        // |[ raygen shader ]|[ hit shader  ]|[ miss shader ]|
        // |                 |               |               |
        // | 0               | 1             | 2             | 3

        if self.accel.is_none() {
            let drawables = self.scene.borrow().cull();
            let mut geometries = vec![];
            for d in &drawables {
                let drawable = d.drawable.borrow();
                geometries.push(drawable.get_geometry().clone());
            }
            self.accel = Some(AccelerationStructure::new(&self.device.borrow(), &mut self.resource_manager.borrow_mut(), &geometries));
        }

        let rt_pipeline_properties = &self.device.borrow().rt_pipeline.properties;
        let handle_size_aligned = Self::aligned_size(
            rt_pipeline_properties.shader_group_handle_size,
            rt_pipeline_properties.shader_group_base_alignment,
        ) as u64;

        let sbt_address = self.shader_binding_table_buffer.borrow().get_buffer_device_address(&self.device.borrow());

        let sbt_raygen_region = vk::StridedDeviceAddressRegionKHR::builder()
            .device_address(sbt_address + 0)
            .size(handle_size_aligned)
            .stride(handle_size_aligned)
            .build();

        let sbt_miss_region = vk::StridedDeviceAddressRegionKHR::builder()
            .device_address(sbt_address + 2 * handle_size_aligned)
            .size(handle_size_aligned)
            .stride(handle_size_aligned)
            .build();

        let sbt_hit_region = vk::StridedDeviceAddressRegionKHR::builder()
            .device_address(sbt_address + 1 * handle_size_aligned)
            .size(handle_size_aligned)
            .stride(handle_size_aligned)
            .build();

        let sbt_call_region = vk::StridedDeviceAddressRegionKHR::default();

        let descriptor_sets = [self.get_descriptor_set().unwrap()];
        let device_ref = self.device.borrow();
        unsafe {
            device_ref.logical_device.cmd_bind_pipeline(
                cmd_buffer,
                vk::PipelineBindPoint::RAY_TRACING_KHR,
                self.pipeline,
            );
            device_ref.logical_device.cmd_bind_descriptor_sets(
                cmd_buffer,
                vk::PipelineBindPoint::RAY_TRACING_KHR,
                self.pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            );
            device_ref.rt_pipeline.pipeline.cmd_trace_rays(
                cmd_buffer,
                &sbt_raygen_region,
                &sbt_miss_region,
                &sbt_hit_region,
                &sbt_call_region,
                512,
                512,
                1,
            );
            //device_ref.logical_device.end_command_buffer(cmd_buffer).unwrap();
        }

        vec![]
    }

    fn get_pipeline(&self) -> &Pipeline {
        todo!()
    }

    fn get_descriptor_set(&self) -> Result<vk::DescriptorSet, &'static str> {
        match self
            .resource_manager
            .borrow_mut()
            .descriptor_set_manager
            .allocate_descriptor_set(&self.descriptor_set_layout) {
            Ok(descriptor_set) => {
                let device_ref = self.device.borrow();
                let accel_structs = [self.accel.as_ref().unwrap().tlas];
                let mut accel_info = vk::WriteDescriptorSetAccelerationStructureKHR::builder()
                    .acceleration_structures(&accel_structs)
                    .build();


                let mut accel_write = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::ACCELERATION_STRUCTURE_KHR)
                    .push_next(&mut accel_info)
                    .build();
                accel_write.descriptor_count = 1;

                let barrier_params = ImageAccess {
                    new_layout: vk::ImageLayout::GENERAL,
                    src_stage: vk::PipelineStageFlags::TOP_OF_PIPE,
                    src_access: vk::AccessFlags::default(),
                    dst_stage: vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR,
                    dst_access: vk::AccessFlags::SHADER_WRITE,
                };
                let image_view = match self.image.borrow_mut().access_view(&device_ref, &barrier_params, None) {
                    Ok(view) => view,
                    Err(msg) => {
                        log::error!("{}", msg);
                        panic!("{}", msg);
                    }
                };

                let image_info = [vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::GENERAL)
                    .image_view(image_view)
                    .build()];

                let image_write = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .image_info(&image_info)
                    .build();

                let buffer_info = [{
                    let buffer_ref = self.color_buffer.borrow();
                    let barrier_params = BufferAccess {
                        src_access: vk::AccessFlags::TRANSFER_WRITE,
                        src_stage: vk::PipelineStageFlags::TRANSFER,
                        dst_access: vk::AccessFlags::SHADER_READ,
                        dst_stage: vk::PipelineStageFlags::RAY_TRACING_SHADER_KHR,
                    };
                    let buffer = buffer_ref.access_buffer(&device_ref, &barrier_params);
                    vk::DescriptorBufferInfo {
                        buffer,
                        range: buffer_ref.size,
                        offset: 0,
                    }
                }];

                let buffers_write = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(2)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(&buffer_info)
                    .build();

                let descr_set_writes = [accel_write, image_write, buffers_write];
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
