use crate::ForwardGraph;

pub struct ModelState {}
pub struct StateIterator {
    graph: ForwardGraph,
    from: f64,
    until: f64,
}

impl StateIterator {
    pub fn new(graph: crate::ForwardGraph, from: f64, until: f64) -> Self {
        let mut graph = graph;
        graph.update_state(from).unwrap();
        Self { graph, from, until }
    }
}

impl Iterator for StateIterator {
    type Item = ModelState;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
