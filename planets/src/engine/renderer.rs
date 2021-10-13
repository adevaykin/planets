use crate::engine::framegraph::FrameGraph;

pub struct Renderer {
    frame_graph: FrameGraph,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            frame_graph: FrameGraph::new(),
        }
    }

    pub fn render(&mut self) {
        self.frame_graph.build();
        self.frame_graph.execute();
    }
}