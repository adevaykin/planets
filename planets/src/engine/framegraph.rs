use ash::vk;
use crate::vulkan::image::{Image, ImageMutRef};
use crate::vulkan::resources::ResourceManagerMutRef;
use crate::engine::viewport::Viewport;

pub enum AttachmentDirection {
    Read,
    Write,
    ReadWrite,
}

pub enum AttachmentSize {
    Absolute(u32,u32),
    Relative(f32,f32),
}

pub struct Attachment {
    name: &'static str,
    size: AttachmentSize,
    format: vk::Format,
    direction: AttachmentDirection,
}

impl Attachment {
    pub fn new(name: &'static str, size: AttachmentSize, format: vk::Format, direction: AttachmentDirection) -> Self {
        Attachment {
            name,
            size,
            format,
            direction,
        }
    }
}

pub trait RenderPass {
    fn get_name(&self) -> &str;
    fn run(&mut self, cmd_buffer: vk::CommandBuffer, attachments: Vec<vk::ImageView>);
    fn get_attachments(&self) -> &Vec<Attachment>;
}

pub struct FrameGraph {
    passes: Vec<Box<dyn RenderPass>>,
    attachments: Vec<ImageMutRef>,
}

impl FrameGraph {
    pub fn new(resource_manager: &ResourceManagerMutRef, viewport: &Viewport) -> Self {

        FrameGraph {
            passes: vec![],
            attachments: vec![resource_manager.borrow_mut().image_attachment(viewport.width, viewport.height, vk::Format::R8G8B8A8_SRGB, "FirstAttachment")],
        }
    }

    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }

    pub fn build(&mut self) {

    }

    pub fn execute(&mut self, cmd_buffer: vk::CommandBuffer) {
        for pass in &mut self.passes {
            let mut attachment_views = vec![];
            for a in &self.attachments {
                attachment_views.push(a.borrow_mut().add_get_view(vk::Format::R8G8B8A8_SRGB));
            }
            pass.run(cmd_buffer, attachment_views);
        }
    }
}

mod tests {
    use crate::engine::framegraph::{FrameGraph, RenderPass, Attachment, AttachmentDirection};
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::vulkan::image::Image;
    use ash::vk;

    struct TestData {
        is_executed: bool,
    }

    struct TestPass {
        test_data: Rc<RefCell<TestData>>,
        attachments: Vec<Attachment>,
    }

    impl TestPass {
        fn new(test_data: &Rc<RefCell<TestData>>) -> Self {
            TestPass {
                test_data: Rc::clone(test_data),
                attachments: vec![]
            }
        }
    }

    impl RenderPass for TestPass {
        fn get_name(&self) -> &str {
            "TestPass"
        }

        fn run(&mut self, cmd_buffer: vk::CommandBuffer, attachments: Vec<vk::ImageView>) {
            self.test_data.borrow_mut().is_executed = true;
        }

        fn get_attachments(&self) -> &Vec<Attachment> {
            &self.attachments
        }
    }

    #[test]
    fn all_passes_executed() {
        let mut graph = FrameGraph::new();

        let test_data1 = Rc::new(RefCell::new(TestData{ is_executed: false }));
        let test_pass1 = Box::new(TestPass::new(&test_data1));
        let test_data2 = Rc::new(RefCell::new(TestData{ is_executed: false }));
        let test_pass2 = Box::new(TestPass::new(&test_data2));

        graph.add_pass(test_pass1);
        graph.add_pass(test_pass2);

        graph.build();
        graph.execute(vk::CommandBuffer::null());

        assert_eq!(test_data1.borrow().is_executed, true);
        assert_eq!(test_data1.borrow().is_executed, true);
    }
}