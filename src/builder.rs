use crate::specification::Deme;
use crate::specification::GenerationTime;
use crate::specification::Graph;
use crate::specification::GraphDefaults;
use crate::specification::MigrationRate;
use crate::specification::Proportion;
use crate::specification::Time;
use crate::specification::TimeUnits;
use crate::specification::UnresolvedDemeHistory;
use crate::specification::UnresolvedEpoch;
use crate::DemesError;

pub struct GraphBuilder {
    graph: Graph,
}

impl GraphBuilder {
    // public API
    pub fn new(
        time_units: TimeUnits,
        generation_time: Option<GenerationTime>,
        defaults: Option<GraphDefaults>,
    ) -> Self {
        Self {
            graph: Graph::new(time_units, generation_time, defaults),
        }
    }

    pub fn new_generations(defaults: Option<GraphDefaults>) -> Self {
        Self {
            graph: Graph::new(TimeUnits::Generations, None, defaults),
        }
    }

    pub fn resolve(self) -> Result<Graph, DemesError> {
        let mut builder = self;
        builder.graph.resolve()?;
        Ok(builder.graph)
    }

    pub fn add_deme(
        &mut self,
        name: &str,
        epochs: Vec<UnresolvedEpoch>,
        history: UnresolvedDemeHistory,
        description: Option<&str>,
    ) {
        let ptr = Deme::new_via_builder(name, epochs, history, description);
        self.graph.add_deme(ptr);
    }

    pub fn add_migration(
        &mut self,
        demes: Option<Vec<String>>,
        source: Option<String>,
        dest: Option<String>,
        rate: Option<MigrationRate>,
        start_time: Option<Time>,
        end_time: Option<Time>,
    ) {
        self.graph
            .add_migration(demes, source, dest, rate, start_time, end_time);
    }

    pub fn add_pulse(
        &mut self,
        sources: Option<Vec<String>>,
        dest: Option<String>,
        time: Option<Time>,
        proportions: Option<Vec<Proportion>>,
    ) {
        self.graph.add_pulse(sources, dest, time, proportions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specification::DemeDefaults;
    use crate::specification::DemeSize;
    use crate::specification::Proportion;

    #[test]
    #[should_panic]
    fn new_builder() {
        let b = GraphBuilder::new(TimeUnits::Generations, None, None);
        b.resolve().unwrap();
    }

    #[test]
    fn add_deme_with_epochs() {
        let mut b = GraphBuilder::new_generations(Some(GraphDefaults::default()));
        let edata = UnresolvedEpoch {
            start_size: Some(DemeSize::try_from(100.0).unwrap()),
            ..Default::default()
        };
        b.add_deme("CEU", vec![edata], UnresolvedDemeHistory::default(), None);
        let _graph = b.resolve().unwrap();
    }

    #[test]
    fn use_proportion_for_proportions() {
        let p = Proportion::try_from(0.5).unwrap();
        let _ = UnresolvedDemeHistory {
            proportions: Some(vec![p, p]),
            ..Default::default()
        };
    }

    #[test]
    fn builder_deme_defaults() {
        let defaults = DemeDefaults {
            epoch: UnresolvedEpoch {
                end_size: Some(DemeSize::try_from(100.).unwrap()),
                ..Default::default()
            },
        };
        let history = UnresolvedDemeHistory {
            defaults,
            ..Default::default()
        };
        let mut b = GraphBuilder::new_generations(None);
        b.add_deme("YRB", vec![], history, None);
        b.resolve().unwrap();
    }
}
