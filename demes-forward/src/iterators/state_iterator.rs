use crate::square_matrix::SquareMatrix;
use crate::time::ForwardTime;
use crate::{CurrentSize, ForwardGraph};

#[derive(Debug)]
pub struct ModelState {
    time: demes::Time,
    forward_time: ForwardTime,
    parental_deme_sizes: Option<Vec<CurrentSize>>,
    offspring_deme_sizes: Option<Vec<CurrentSize>>,
    ancestry_proportions: Option<SquareMatrix>,
    name_to_index: std::collections::HashMap<String, usize>,
}

impl ModelState {
    fn new(graph: &ForwardGraph) -> Self {
        let forward_time = graph.last_time_updated().unwrap();
        let time = graph.time_to_backward(forward_time).unwrap().unwrap();
        let parental_deme_sizes = graph.parental_deme_sizes().map(|x| x.to_vec());
        let offspring_deme_sizes = graph.offspring_deme_sizes().map(|x| x.to_vec());
        let ancestry_proportions = match offspring_deme_sizes {
            None => None,
            Some(_) => {
                let mut ancestry_proportions = SquareMatrix::zeros(graph.num_demes_in_model());
                for i in 0..graph.num_demes_in_model() {
                    if let Some(a) = graph.ancestry_proportions(i) {
                        let r = ancestry_proportions.row_mut(i);
                        r.copy_from_slice(a);
                    }
                }
                Some(ancestry_proportions)
            }
        };
        todo!("need a deme id -> ancestry_proportion map");
        Self {
            time,
            forward_time,
            parental_deme_sizes,
            offspring_deme_sizes,
            ancestry_proportions,
        }
    }

    pub fn time(&self) -> demes::Time {
        self.time
    }

    pub fn forward_time(&self) -> ForwardTime {
        self.forward_time
    }

    pub fn parental_deme_sizes(&self) -> Option<&[CurrentSize]> {
        self.parental_deme_sizes.as_deref()
    }

    pub fn offspring_deme_sizes(&self) -> Option<&[CurrentSize]> {
        self.offspring_deme_sizes.as_deref()
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
                    Some(ModelState::new(&self.graph))
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
