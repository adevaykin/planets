use std::rc::Rc;

use ash::vk;

use super::device::DeviceMutRef;

pub struct Sampler {
    device: DeviceMutRef,
    pub sampler: vk::Sampler,
}

impl Sampler {
    pub fn new(device: &DeviceMutRef) -> Sampler {
        let create_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: device.borrow().physical_props.limits.max_sampler_anisotropy,
            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            min_lod: 0.0,
            max_lod: 0.0,
            mip_lod_bias: 0.0,
            ..Default::default()
        };

        let sampler = unsafe {
            device
                .borrow()
                .logical_device
                .create_sampler(&create_info, None)
                .expect("Failed to create sampler")
        };

        Sampler {
            device: Rc::clone(device),
            sampler,
        }
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.device
                .borrow()
                .logical_device
                .destroy_sampler(self.sampler, None);
        }
    }
}
