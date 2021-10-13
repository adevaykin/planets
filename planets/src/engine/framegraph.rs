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
    name: String,
    size: AttachmentSize,
}

pub trait RenderPass {
    fn get_name(&self) -> &str;
    fn run(&mut self);
    fn get_attachments(&self) -> &Vec<(Attachment,AttachmentDirection)>;
}

pub struct FrameGraph {
    passes: Vec<Box<dyn RenderPass>>,
}

impl FrameGraph {
    pub fn new() -> Self {
        FrameGraph {
            passes: vec![],
        }
    }

    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }

    pub fn build(&mut self) {

    }

    pub fn execute(&mut self) {
        for pass in &mut self.passes {
            pass.run();
        }
    }
}

mod tests {
    use crate::engine::framegraph::{FrameGraph, RenderPass, Attachment, AttachmentDirection};
    use std::cell::RefCell;
    use std::rc::Rc;

    struct TestData {
        is_executed: bool,
    }

    struct TestPass {
        test_data: Rc<RefCell<TestData>>,
        attachments: Vec<(Attachment,AttachmentDirection)>,
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

        fn run(&mut self) {
            self.test_data.borrow_mut().is_executed = true;
        }

        fn get_attachments(&self) -> &Vec<(Attachment,AttachmentDirection)> {
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
        graph.execute();

        assert_eq!(test_data1.borrow().is_executed, true);
        assert_eq!(test_data1.borrow().is_executed, true);
    }
}