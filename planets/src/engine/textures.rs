use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

use crate::vulkan::device::{Device, DeviceMutRef};
use crate::vulkan::image::image::{Image, ImageMutRef};
use crate::vulkan::resources::{ResourceManager, ResourceManagerMutRef};

const INVALID_IMAGE_PATH: &str = "assets/textures/invalid.png";

pub type TextureManagerMutRef = Rc<RefCell<TextureManager>>;

pub trait Uploadable {
    fn upload(&self, device: &Device, resource_manager: &mut ResourceManager);
}

// TODO: move to dedicated module
struct UpdateCollection<'a> {
    pending: Vec<&'a dyn Uploadable>,
}

impl UpdateCollection<'_> {
    pub fn new() -> Self {
        UpdateCollection {
            pending: vec![]
        }
    }

    pub fn add()

    pub fn apply(&mut self, device: &Device) {
        for p in &self.pending {
            p.upload(device);
        }

        self.pending.clear();
    }
}

pub struct TextureManager {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    loaded: HashMap<String, ImageMutRef>,
    pending_uploads: UpdataCollection,
}

impl TextureManager {
    pub fn new(device: &DeviceMutRef, resource_manager: &ResourceManagerMutRef) -> TextureManager {
        let invalid_image = Rc::new(RefCell::new(
            Image::from_file(
                device,
                INVALID_IMAGE_PATH,
            )
            .unwrap(),
        ));
        let mut loaded: HashMap<String, ImageMutRef> = HashMap::new();
        loaded.insert(INVALID_IMAGE_PATH.to_string(), invalid_image);

        let mut pending_uploads = UpdateCollection::new();
        pending_upload

        TextureManager {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            loaded,
            pending_uploads,
        }
    }

    pub fn get_texture(&mut self, path: &str) -> ImageMutRef {
        let existing = self.loaded.get(&path.to_string());
        if existing.is_some() {
            return Rc::clone(existing.unwrap());
        } else {
            let new_image =
                Image::from_file(&self.device, &mut self.resource_manager.borrow_mut(), path);
            if new_image.is_ok() {
                self.loaded
                    .insert(path.to_string(), Rc::new(RefCell::new(new_image.unwrap())));
                return self.get_texture(path);
            }
            return Rc::clone(self.loaded.get(INVALID_IMAGE_PATH).unwrap());
        }
    }

    pub fn apply_changes(&mut self) {

    }
}
