use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

use crate::vulkan::device::{DeviceMutRef};
use crate::vulkan::image::image::{Image, ImageMutRef};
use crate::vulkan::resources::{ResourceManagerMutRef};

const INVALID_IMAGE_PATH: &str = "assets/textures/invalid.png";

pub type TextureManagerMutRef = Rc<RefCell<TextureManager>>;

pub struct TextureManager {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    loaded: HashMap<String, ImageMutRef>,
    pending_uploads: HashMap<String,ImageMutRef>,
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
        let mut pending_uploads: HashMap<String, ImageMutRef> = HashMap::new();
        pending_uploads.insert(INVALID_IMAGE_PATH.to_string(), invalid_image);

        TextureManager {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            loaded: HashMap::new(),
            pending_uploads,
        }
    }

    pub fn get_texture(&mut self, path: &str) -> &ImageMutRef {
        let path_string = path.to_string();

        if let Some(existing) = self.loaded.get(&path_string) {
            return existing;
        }

        return if let Ok(new_image) = Image::from_file(&self.device, path) {
            self.pending_uploads
                .insert(path_string.clone(), Rc::new(RefCell::new(new_image)));
            self.pending_uploads.get(&path_string).unwrap()
        } else {
            &self.loaded.get(INVALID_IMAGE_PATH).unwrap()
        }
    }

    pub fn upload_pending(&mut self) {
        for (key, image) in &self.pending_uploads {
            if let Ok(()) = image.borrow_mut().upload(&self.device.borrow(), &mut self.resource_manager.borrow_mut()) {
                self.loaded.insert(key.clone(), Rc::clone(image));
            } else {
                log::error!("Failed to upload image {}", key);
            }
        }

        self.pending_uploads.clear();
    }
}
