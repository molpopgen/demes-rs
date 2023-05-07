use demes::DemeSize;

use crate::time::ForwardTime;
use crate::ForwardGraph;

pub struct ModelState {
    time: demes::Time,
    forward_time: ForwardTime,
    parental_deme_sizes: Option<Vec<DemeSize>>,
    offspring_deme_sizes: Option<Vec<DemeSize>>,
    // TODO: we need something about ancestry proportions
    // here...
}

impl ModelState {
    fn new() -> Self {
        todo!("implement ModelState::new()");
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
            Some(time) => {
                println!("{time:?}");
                if (time.value() >= self.iterate_from) && (time.value() <= self.iterate_until) {
                    self.graph.update_state(time).unwrap();
                    Some(ModelState::new())
                } else {
                    None
                }
            }
            None => {
                println!("none??");
                None
            }
        }
    }
}
