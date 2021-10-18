use std::rc::Rc;

use crate::vulkan::image::{Image, ImageMutRef};

pub struct Material {
    pub albedo_map: Option<ImageMutRef>,
    pub normal_map: Option<ImageMutRef>,
    pub roughness_map: Option<ImageMutRef>,
}

impl Material {
    pub fn new() -> Material {
        Material {
            albedo_map: None,
            normal_map: None,
            roughness_map: None,
        }
    }
}
