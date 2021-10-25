use std::cell::RefCell;
use std::rc::Rc;

pub type ViewportMutRef = Rc<RefCell<Viewport>>;

pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

impl Viewport {
    pub fn new(width: u32, height: u32) -> Self {
        Viewport {
            width,
            height
        }
    }

    pub fn update(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}
