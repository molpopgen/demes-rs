use crate::time::ForwardTime;
use crate::ForwardGraph;

pub struct ModelState {}

impl ModelState {
    fn new() -> Self {
        todo!()
    }
}

pub struct StateIterator {
    graph: ForwardGraph,
    iterate_from: f64,
    iterate_until: f64,
    time_iterator: Box<dyn Iterator<Item = ForwardTime>>,
}

impl StateIterator {
    pub fn new(graph: crate::ForwardGraph, iterate_from: f64, iterate_until: f64) -> Self {
        let mut graph = graph;
        graph.update_state(iterate_from).unwrap();
        let time_iterator = Box::new(graph.time_iterator());
        let time_iterator2 = Box::new(graph.time_iterator());
        for t in time_iterator2 {
            println!("time = {t:?}");
        }
        Self {
            graph,
            iterate_from,
            iterate_until,
            time_iterator,
        }
    }
}

impl Iterator for StateIterator {
    type Item = ModelState;

    fn next(&mut self) -> Option<Self::Item> {
        match self.time_iterator.next() {
            Some(time)
                if (time.value() >= self.iterate_from) && (time.value() <= self.iterate_until) =>
            {
                self.graph.update_state(time).unwrap();
                Some(ModelState::new())
            }
            Some(_) => None,
            None => None,
        }
    }
}
