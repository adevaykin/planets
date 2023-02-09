use std::rc::Rc;

use ash::vk;
use ash::vk::Handle;

use super::device::DeviceMutRef;
use std::cell::RefCell;
use crate::vulkan::debug::DebugResource;

pub type FramebufferMutRef = Rc<RefCell<Framebuffer>>;

pub struct Framebuffer {
    device: DeviceMutRef,
    pub framebuffer: vk::Framebuffer,
    label: String
}

impl Framebuffer {
    pub fn new(
        device: &DeviceMutRef,
        width: u32,
        height: u32,
        attachment_views: &Vec<vk::ImageView>,
        render_pass: vk::RenderPass,
        label: &str,
    ) -> Framebuffer {
        let create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            render_pass,
            attachment_count: attachment_views.len() as u32,
            p_attachments: attachment_views.as_ptr(),
            width,
            height,
            layers: 1,
            ..Default::default()
        };

        let framebuffer = unsafe {
            device
                .borrow()
                .logical_device
                .create_framebuffer(&create_info, None)
                .expect("Failed to create framebuffer.")
        };

        Framebuffer {
            device: Rc::clone(device),
            framebuffer,
            label: String::from(label),
        }
    }
}

impl DebugResource for Framebuffer {
    fn get_type(&self) -> vk::ObjectType {
        vk::ObjectType::FRAMEBUFFER
    }

    fn get_handle(&self) -> u64 {
        self.framebuffer.as_raw()
    }

    fn get_label(&self) -> &String {
        &self.label
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .borrow()
                .logical_device
                .destroy_framebuffer(self.framebuffer, None);
        }
    }
}
