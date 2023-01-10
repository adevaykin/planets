use std::cell::RefCell;
use std::rc::Rc;
use crate::engine::geometry::Geometry;
use crate::engine::material::Material;
use crate::engine::scene::graph::SceneGraph;
use crate::engine::scene::node::{Node, NodeContent};
use crate::vulkan::drawable::{Drawable, DrawType};
use cgmath as cgm;
use cgmath::SquareMatrix;
use crate::vulkan::entry::Entry;

pub fn build_scene(vulkan: &Entry, scene: &mut SceneGraph) {
    let transform_node = Rc::new(RefCell::new(
        Node::with_content(
            NodeContent::Transform(cgm::Matrix4::from_scale(10.0))
        )
    ));

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
}
