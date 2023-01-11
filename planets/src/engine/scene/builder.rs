use std::cell::RefCell;
use std::rc::Rc;
use crate::engine::geometry::Geometry;
use crate::engine::material::Material;
use crate::engine::scene::graph::SceneGraph;
use crate::engine::scene::node::{Node, NodeContent, UpdateCallResult};
use crate::vulkan::drawable::{Drawable, DrawType};
use cgmath as cgm;
use cgmath::SquareMatrix;
use crate::vulkan::entry::Entry;
use crate::world::loader::ModelLoader;

pub fn build_scene(vulkan: &Entry, scene: &mut SceneGraph, model_loader: &mut ModelLoader) {
    let transform_node = Rc::new(RefCell::new(
        Node::with_content(
            NodeContent::Transform(cgm::Matrix4::from_scale(100.0))
        )
    ));

    transform_node.borrow_mut().update_call = Some(Box::new(|node, game_loop| {
        let rotate = cgm::Matrix4::from_angle_y(cgm::Deg(game_loop.borrow().get_prev_frame_time().as_millis() as f32 * 0.10));
        let new_transform = rotate * node.get_transform();
        UpdateCallResult {
            transform: Some(new_transform),
            pre_update_action: None,
        }
    }));

    let box_geometry = Geometry::quad(&mut vulkan.get_resource_manager().borrow_mut());
    let box_material = Material::new();
    let box_drawable = Rc::new(RefCell::new(Drawable::new(
        &mut vulkan.get_resource_manager().borrow_mut(),
        DrawType::Opaque,
        box_geometry,
        box_material
    )));
    let box_content = NodeContent::Drawable(box_drawable.clone());
    let box_node = Rc::new(RefCell::new(Node::with_content(box_content)));
    let box_instance = Drawable::create_instance(&box_drawable);
    box_node.borrow_mut().add_child(Rc::new(RefCell::new(Node::with_content(NodeContent::DrawableInstance(box_instance)))));
    transform_node.borrow_mut().add_child(box_node);
    scene.root.add_child(transform_node);

    let model3 = model_loader.load_gltf("assets/gltf/starship/starship.gltf");
    scene.root.add_child(Rc::clone(&model3));
}
