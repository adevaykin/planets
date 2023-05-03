use std::cell::RefCell;
use std::rc::Rc;
use crate::engine::scene::graph::SceneGraph;
use crate::engine::scene::node::{Node, NodeContent, UpdateCallResult};
use cgmath as cgm;
use cgmath::SquareMatrix;
use crate::world::loader::ModelLoader;

pub fn build_scene(scene: &mut SceneGraph, model_loader: &mut ModelLoader) {
    match model_loader.load_gltf("assets/gltf/ao/ao.gltf") {
        Ok(model) => {
            let instance = model.borrow_mut().spawn_instance();
            scene.root.add_child(Rc::clone(&model));
            let mut transform_node = Node::with_content(NodeContent::Transform(cgm::Matrix4::from_scale(0.2)));
            transform_node.add_child(instance);
            transform_node.update_call = Some(Box::new(|node, gameloop| {
                let transform = if let NodeContent::Transform(current_transform) = node.content {
                    current_transform
                        * cgm::Matrix4::from_angle_y(cgm::Deg((gameloop.get_total_elapsed().as_millis() as f32 * 0.005).sin()))
                        //* cgm::Matrix4::from_angle_x(cgm::Deg(gameloop.get_prev_frame_time().as_millis() as f32 * 0.05))
                } else {
                    cgm::Matrix4::identity()
                };
                UpdateCallResult {
                    transform: Some(transform),
                    pre_update_action: None,
                }
            }));
            scene.root.add_child(Rc::new(RefCell::new(transform_node)));

        },
        Err(str) => log::error!("{}", str),
    };
}
