use crate::time::ForwardTime;
use crate::DemeSizeAt;
use crate::DemesForwardError;
use crate::ForwardGraph;

pub struct DemeSizeHistory {
    graph: ForwardGraph,
    deme_index: usize,
    forward_model_start_time: f64,
    time_iterator: Box<dyn Iterator<Item = ForwardTime>>,
}

impl DemeSizeHistory {
    pub fn new(
        graph: ForwardGraph,
        deme_index: usize,
        forward_model_start_time: f64,
    ) -> Result<Self, DemesForwardError> {
        let mut graph = graph;
        // NOTE: we need to maually update
        // the internal state to the first generation
        // in case we are cloning from a graph that
        // has been treated as mutable.
        graph.update_state(0.0).unwrap();
        let time_iterator = Box::new(graph.time_iterator());
        Ok(Self {
            graph,
            deme_index,
            forward_model_start_time,
            time_iterator,
        })
    }
}

impl Iterator for DemeSizeHistory {
    type Item = crate::DemeSizeAt;

    fn next(&mut self) -> Option<Self::Item> {
        match self.time_iterator.next() {
            Some(forward_time) => {
                self.graph.update_state(forward_time).unwrap();
                let size = self.graph.parental_deme_sizes().unwrap()[self.deme_index];
                let time = self.forward_model_start_time;
                self.forward_model_start_time -= 1.0;
                let item = DemeSizeAt {
                    time: time.try_into().unwrap(),
                    forward_time,
                    size,
                };
                Some(item)
            }
            None => None,
        }
    }
}
