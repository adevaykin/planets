use std::cell::RefCell;
use std::rc::Rc;

use cgmath as cgm;
use cgmath::prelude::*;

use crate::engine::lights::{LightManager, LightManagerMutRef};
use crate::engine::scene::node::{Node, NodeContent};
use crate::vulkan::device::{Device, DeviceMutRef};
use crate::vulkan::drawable::DrawableHash;
use crate::vulkan::resources::{ResourceManagerMutRef};
use std::collections::HashSet;
use crate::engine::gameloop::GameLoopMutRef;
use crate::engine::models::ModelData;
use crate::engine::scene::drawlist::{DrawList, DrawListMutRef};

pub const UP: cgm::Vector3<f32> = cgm::Vector3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};

pub type SceneGraphMutRef = Rc<RefCell<SceneGraph>>;

pub struct SceneGraph {
    light_manager: LightManagerMutRef, // TODO: extract light manager from SceneGraph?
    model_data: ModelData,
    pub root: Node,
    draw_list: DrawListMutRef, // TODO: should not be part of SceneGraph - cull() should return new to draw list
}

impl SceneGraph {
    pub fn new_mut_ref(device: &DeviceMutRef, resource_manager: &ResourceManagerMutRef) -> SceneGraphMutRef {
        Rc::new(RefCell::new(SceneGraph::new(device, resource_manager)))
    }

    pub fn new(device: &DeviceMutRef, resource_manager: &ResourceManagerMutRef) -> SceneGraph {
        let root = Node::new();
        let light_manager = Rc::new(RefCell::new(LightManager::new(&mut resource_manager.borrow_mut())));

        let mut scene = SceneGraph {
            model_data: ModelData::new(resource_manager),
            root,
            light_manager,
            draw_list: DrawList::new_mut_ref(device)
        };

        let mut light_transform_node = Node::with_content(NodeContent::Transform(
            cgm::Matrix4::from_translation(cgm::Vector3::new(0.0, 0.0, -10.0)),
        ));
        let light_node = Rc::new(RefCell::new(Node::with_content(NodeContent::Light(
            LightManager::create_light(&scene.light_manager),
        ))));
        light_transform_node.add_child(light_node);
        let light_transform_node = Rc::new(RefCell::new(light_transform_node));
        scene.root.add_child(light_transform_node);

        scene
    }

    pub fn update(&mut self, device: &Device, gameloop: &GameLoopMutRef) {
        let identity = cgm::Matrix4::identity();
        self.root.update(device, gameloop, &identity, &mut self.model_data);
        //self.light_manager.borrow_mut().update(device);
        self.model_data.update(device);
    }

    pub fn cull(&self) -> HashSet<DrawableHash> {
        let mut drawables = HashSet::new();
        self.root.cull(&mut drawables);

        drawables
    }

    pub fn get_model_data(&self) -> &ModelData {
        &self.model_data
    }

    pub fn get_light_manager(&self) -> &LightManagerMutRef {
        &self.light_manager
    }

    pub fn get_draw_list(&self) -> &DrawListMutRef {
        &self.draw_list
    }
}
