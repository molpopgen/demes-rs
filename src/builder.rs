use crate::specification::Graph;
use crate::specification::TimeUnits;
use crate::DemesError;

pub struct GraphBuilder {
    graph: Graph,
}

impl GraphBuilder {
    pub fn new(time_units: TimeUnits) -> Self {
        Self {
            graph: Graph::new_from_time_units(time_units),
        }
    }

    pub fn resolve(self) -> Result<Graph, DemesError> {
        let mut builder = self;
        builder.graph.resolve()?;
        Ok(builder.graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn new_builder() {
        let b = GraphBuilder::new(TimeUnits::GENERATIONS);
        b.resolve().unwrap();
    }
}
