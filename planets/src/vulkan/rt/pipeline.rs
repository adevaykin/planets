use ash::extensions::khr::RayTracingPipeline;
use ash::{vk};

pub struct RtPipeline {
    pub pipeline: RayTracingPipeline,
    pub properties: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR,
}

impl RtPipeline {
    pub fn new(instance: &ash::Instance, physical_device: &vk::PhysicalDevice, logical_device: &ash::Device) -> Self {
        let mut properties = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();

        {
            let mut physical_device_properties2 = vk::PhysicalDeviceProperties2::builder()
                .push_next(&mut properties)
                .build();

            unsafe {
                instance
                    .get_physical_device_properties2(*physical_device, &mut physical_device_properties2);
            }
        }

        let pipeline = RayTracingPipeline::new(instance, logical_device);

        Self {
            pipeline,
            properties,
        }
    }
}