use crate::engine::lights::{Light, LightManagerMutRef};
use crate::util::math;
use crate::vulkan::device::Device;
use crate::vulkan::drawable::{Drawable, DrawableHash, DrawableInstanceMutRef, DrawableMutRef};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use cgmath as cgm;
use crate::engine::gameloop::GameLoopMutRef;

pub type NodeMutRef = Rc<RefCell<Node>>;

pub enum PreUpdateAction {
    NONE,
    DELETE,
}

pub type NodeUpdateCall = Box<dyn Fn(&Node, &GameLoopMutRef) -> UpdateCallResult>;

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
        if let NodeContent::DrawableInstance(d) = &self.content {
            drawables.insert(DrawableHash::new(&d.borrow().drawable.upgrade().unwrap()));
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

        for c in &mut self.children {
            c.borrow_mut().remove_child(child_to_remove);
        }
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

    pub fn create_instance(&self) -> NodeMutRef {
        let mut instance_node = Node::new();

        instance_node.content = match &self.content {
            NodeContent::Drawable(d) => NodeContent::DrawableInstance(Drawable::create_instance(d)),
            _ => self.content.clone(),
        };

        for child in &self.children {
            let child_instance = child.borrow().create_instance();
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
        gameloop: &GameLoopMutRef,
        transform: &cgm::Matrix4<f32>,
    ) {
        if self.update_call.is_some() {
            let update_call = self.update_call.as_ref().unwrap();
            let update_call_result = update_call(&self, gameloop);
            if update_call_result.transform.is_some() {
                self.content = NodeContent::Transform(update_call_result.transform.unwrap());
            }
            if update_call_result.pre_update_action.is_some() {
                self.pre_update_action = update_call_result.pre_update_action.unwrap();
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
                d.borrow_mut().update(device, &next_transform);
            }
            _ => {}
        }

        let mut nodes_to_delete = vec![];
        for child in &mut self.children {
            match &child.borrow().pre_update_action {
                PreUpdateAction::NONE => {}
                PreUpdateAction::DELETE => {
                    nodes_to_delete.push(Rc::clone(child));
                    continue;
                }
            }
            child
                .borrow_mut()
                .update(device, gameloop, &next_transform);
        }

        for node in nodes_to_delete {
            self.remove_child(&node);
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
