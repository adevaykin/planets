use std::cell::RefCell;
use std::rc::Rc;

use std::collections::HashMap;

use crate::vulkan::device::DeviceMutRef;
use crate::vulkan::image::Image;
use crate::vulkan::resources::ResourceManagerMutRef;

const INVALID_IMAGE_PATH: &str = "assets/textures/invalid.png";

pub type TextureManagerMutRef = Rc<RefCell<TextureManager>>;

pub struct TextureManager {
    device: DeviceMutRef,
    resource_manager: ResourceManagerMutRef,
    loaded: HashMap<String, Rc<Image>>,
}

impl TextureManager {
    pub fn new(device: &DeviceMutRef, resource_manager: &ResourceManagerMutRef) -> TextureManager {
        let invalid_image = Rc::new(
            Image::from_file(
                device,
                &mut resource_manager.borrow_mut(),
                INVALID_IMAGE_PATH,
            )
            .unwrap(),
        );
        let mut loaded: HashMap<String, Rc<Image>> = HashMap::new();
        loaded.insert(INVALID_IMAGE_PATH.to_string(), invalid_image);

        TextureManager {
            device: Rc::clone(device),
            resource_manager: Rc::clone(resource_manager),
            loaded,
        }
    }

    pub fn get_texture(&mut self, path: &str) -> Rc<Image> {
        let existing = self.loaded.get(&path.to_string());
        if existing.is_some() {
            return Rc::clone(existing.unwrap());
        } else {
            let new_image =
                Image::from_file(&self.device, &mut self.resource_manager.borrow_mut(), path);
            if new_image.is_ok() {
                self.loaded
                    .insert(path.to_string(), Rc::new(new_image.unwrap()));
                return self.get_texture(path);
            }
            return Rc::clone(self.loaded.get(INVALID_IMAGE_PATH).unwrap());
        }
    }
}
