use alloc::rc::Rc;
use std::cell::RefCell;
use ash::vk;
use crate::vulkan::device::Device;
use crate::vulkan::mem::{AllocatedBufferMutRef, VecBufferData};
use crate::vulkan::resources::manager::ResourceManager;

pub struct DrawableMemDescr {
    pub vertex_buf_addr: u64,
    pub index_buf_addr: u64,
}

pub struct ObjectDescriptions {
    is_dirty: bool,
    descriptions: Vec<DrawableMemDescr>,
    ssbo: Option<AllocatedBufferMutRef>,
}

pub type ObjectDescriptionsMutRef = Rc<RefCell<ObjectDescriptions>>;

impl ObjectDescriptions {
    pub fn new() -> Self {
        Self {
            is_dirty: false,
            descriptions: vec![],
            ssbo: None,
        }
    }

    pub fn add_object(&mut self, descr: DrawableMemDescr) {
        self.descriptions.push(descr);
        self.is_dirty = true;
    }

    pub fn update(&mut self, resource_manager: &mut ResourceManager) {
        if self.is_dirty {
            let data = VecBufferData::new(&self.descriptions);
            self.ssbo = Some(resource_manager.buffer_with_staging(
                &data,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                "ObjectDescriptions",
            ));

            self.is_dirty = false;
        }
    }

    pub fn get_ssbo(&self) -> &AllocatedBufferMutRef {
        &self.ssbo.as_ref().expect("ObjectDescriptions SSBO does not exist yet.")
    }
}