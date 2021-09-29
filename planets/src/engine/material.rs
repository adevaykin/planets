use std::rc::Rc;

use crate::vulkan::image::Image;

pub struct Material {
    pub albedo_map: Option<Rc<Image>>,
    pub normal_map: Option<Rc<Image>>,
    pub roughness_map: Option<Rc<Image>>,
}

impl Material {
    pub fn new() -> Material {
        Material { albedo_map: None, normal_map: None, roughness_map: None }
    }
}