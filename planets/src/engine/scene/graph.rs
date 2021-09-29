use std::rc::Rc;
use std::cell::RefCell;

use cgmath as cgm;
use cgmath::prelude::*;

use crate::engine::lights::{LightManager,LightManagerMutRef};
use crate::engine::timer::TimerMutRef;
use crate::vulkan::drawable::DrawableHash;
use crate::vulkan::device::Device;
use crate::vulkan::resources::ResourceManager;
use std::collections::HashSet;
use crate::engine::scene::node::{Node,NodeContent};

pub const UP: cgm::Vector3<f32> = cgm::Vector3{ x: 0.0, y: 1.0, z: 0.0 };

pub type SceneGraphMutRef = Rc<RefCell<SceneGraph>>;

pub struct SceneGraph {
    pub light_manager: LightManagerMutRef, // TODO: extract light manager from SceneGraph?
    pub root: Node,
}

impl SceneGraph {
    pub fn new(resource_manager: &mut ResourceManager) -> SceneGraph {
        let root = Node::new();
        let light_manager = Rc::new(RefCell::new(LightManager::new(resource_manager)));

        let mut scene = SceneGraph { root, light_manager };

        let mut light_transform_node = Node::with_content(NodeContent::Transform(cgm::Matrix4::from_translation(cgm::Vector3::new(0.0, 0.0, -10.0))));
        let light_node = Rc::new(RefCell::new(Node::with_content(NodeContent::Light(LightManager::create_light(&scene.light_manager)))));
        light_transform_node.add_child(light_node);
        let light_transform_node = Rc::new(RefCell::new(light_transform_node));
        scene.root.add_child(light_transform_node);

        scene
    }

    pub fn update(&mut self, device: &Device, frame_num: usize, timer: &TimerMutRef) {
        let identity = cgm::Matrix4::identity();
        self.root.update(device, frame_num, timer, &self.light_manager, &identity);

        self.light_manager.borrow_mut().update(device, frame_num);
    }

    pub fn cull(&self) -> HashSet<DrawableHash> {
        let mut drawables = HashSet::new();
        self.root.cull(&mut drawables);

        drawables
    }
}
