use crate::engine::lights::{Light};
use crate::util::math;
use crate::vulkan::device::Device;
use crate::vulkan::drawable::{Drawable, DrawableHash, DrawableInstanceMutRef, DrawableMutRef};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use cgmath as cgm;
use crate::engine::gameloop::{GameLoop, GameLoopMutRef};
use crate::engine::models::{ModelData, ModelDataSSBOInterface};

pub type NodeMutRef = Rc<RefCell<Node>>;

pub enum PreUpdateAction {
    NONE,
    DELETE,
}

pub type NodeUpdateCall = Box<dyn Fn(&Node, &GameLoop) -> UpdateCallResult>;

pub struct UpdateCallResult {
    pub transform: Option<cgm::Matrix4<f32>>,
    pub pre_update_action: Option<PreUpdateAction>,
}

#[derive(Clone)]
pub enum NodeContent {
    None,
    Transform(cgm::Matrix4<f32>),
    Drawable(DrawableMutRef),
    DrawableInstance(DrawableInstanceMutRef),
    Light(Light),
}

pub struct Node {
    pub pre_update_action: PreUpdateAction,
    pub content: NodeContent,
    children: Vec<NodeMutRef>,
    pub update_call: Option<NodeUpdateCall>,
}

impl Node {
    pub fn new() -> Node {
        Node::with_content(NodeContent::None)
    }

    pub fn with_content(content: NodeContent) -> Node {
        Node {
            pre_update_action: PreUpdateAction::NONE,
            content,
            children: vec![],
            update_call: None,
        }
    }

    pub fn cull(&self, drawables: &mut HashSet<DrawableHash>) {
        if let NodeContent::DrawableInstance(instance) = &self.content {
            if let Some(drawable) = &instance.borrow().drawable.upgrade() {
                drawables.insert(DrawableHash::new(drawable));
            } else {
                log::error!("Failed to upgrade instance to drawable");
            }
        }

        for c in &self.children {
            c.borrow().cull(drawables);
        }
    }

    pub fn add_child(&mut self, child: NodeMutRef) {
        self.children.push(child);
    }

    pub fn remove_child(&mut self, child_to_remove: &NodeMutRef) {
        self.children
            .retain(|child| !Node::children_equal(child, child_to_remove));
    }

    pub fn get_mut_light(&mut self) -> &mut Light {
        match &mut self.content {
            NodeContent::Light(l) => l,
            _ => panic!("Unable to get Light from node having another content type"),
        }
    }

    pub fn get_transform(&self) -> &cgm::Matrix4<f32> {
        match &self.content {
            NodeContent::Transform(t) => t,
            _ => panic!("Unable to get Transform from node having another content type"),
        }
    }

    pub fn spawn_instance(&self) -> NodeMutRef {
        let mut instance_node = Node::new();

        instance_node.content = match &self.content {
            NodeContent::Drawable(d) => NodeContent::DrawableInstance(Drawable::create_instance(d)),
            _ => self.content.clone(),
        };

        for child in &self.children {
            let child_instance = child.borrow().spawn_instance();
            instance_node.add_child(child_instance);
        }

        Rc::new(RefCell::new(instance_node))
    }

    fn children_equal(child1: &NodeMutRef, child2: &NodeMutRef) -> bool {
        Rc::ptr_eq(child1, child2)
    }

    pub fn update(
        &mut self,
        device: &Device,
        gameloop: &GameLoop,
        transform: &cgm::Matrix4<f32>,
        model_data: &mut ModelData,
    ) {
        if let Some(update_call) = self.update_call.as_ref() {
            let update_call_result = update_call(&self, gameloop);
            if let Some(transform) = update_call_result.transform {
                self.content = NodeContent::Transform(transform);
            }
            if let Some(pre_update_action) = update_call_result.pre_update_action {
                self.pre_update_action = pre_update_action;
            }
        }

        let next_transform = match &self.content {
            NodeContent::Transform(t) => transform * t,
            _ => *transform,
        };

        match &mut self.content {
            NodeContent::Light(l) => {
                l.position = math::position_from_transform(&next_transform);
                l.apply();
            }
            NodeContent::DrawableInstance(d) => {
                let new_model_data = ModelDataSSBOInterface{ transform: next_transform };
                model_data.set_data_for(d.borrow().get_instance_id() as usize, &new_model_data);
            }
            _ => {}
        }

        self.children.retain(|child| {
            match child.borrow().pre_update_action {
                PreUpdateAction::DELETE => false,
                _ => true,
            }
        });

        for child in &mut self.children {
            child.borrow_mut()
                .update(device, gameloop, &next_transform, model_data);
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if let NodeContent::DrawableInstance(instance) = &self.content {
            instance.borrow_mut().destroy();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::Node;

    #[test]
    fn node_add_child() {
        let mut node = Node::new();
        assert_eq!(node.children.len(), 0);

        let child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));

        node.add_child(child1);
        assert_eq!(node.children.len(), 1);

        node.add_child(child2);
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn node_remove_child() {
        let mut node = Node::new();

        let mut child1 = Rc::new(RefCell::new(Node::new()));
        let child2 = Rc::new(RefCell::new(Node::new()));
        let child3 = Rc::new(RefCell::new(Node::new()));
        child1.borrow_mut().add_child(Rc::clone(&child3));
        node.add_child(Rc::clone(&child1));
        node.add_child(Rc::clone(&child2));
        node.add_child(Rc::clone(&child3));

        node.remove_child(&child3);
        assert_eq!(node.children.len(), 2);
        assert_eq!(child1.borrow().children.len(), 0);
        assert_eq!(Rc::ptr_eq(&node.children[0], &child1), true);
        assert_eq!(Rc::ptr_eq(&node.children[1], &child2), true);

        node.remove_child(&child1);
        assert_eq!(node.children.len(), 1);
        assert_eq!(Rc::ptr_eq(&node.children[0], &child2), true);

        node.remove_child(&child2);
        assert_eq!(node.children.len(), 0);
    }
}
