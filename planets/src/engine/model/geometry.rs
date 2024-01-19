use alloc::rc::Rc;
use std::cell::RefCell;

pub struct OwnedGeometry {
    pub vertex_offset: usize,
    pub vertex_count: usize,
    pub index_offset: usize,
    pub index_count: usize,
}

pub type OwnedGeometryMutRef = Rc<RefCell<OwnedGeometry>>;
