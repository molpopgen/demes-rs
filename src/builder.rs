use crate::specification::Deme;
use crate::specification::DemeHistory;
use crate::specification::EpochData;
use crate::specification::Graph;
use crate::specification::MigrationRate;
use crate::specification::Time;
use crate::specification::TimeUnits;
use crate::DemesError;

pub struct GraphBuilder {
    graph: Graph,
}

impl GraphBuilder {
    // public API
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

    pub fn add_deme(
        &mut self,
        name: &str,
        epochs: Vec<EpochData>,
        history: DemeHistory,
        description: Option<&str>,
    ) -> Result<(), DemesError> {
        let ptr = Deme::new_via_builder(name, epochs, history, description)?;
        self.graph.add_deme(ptr);
        Ok(())
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
        let b = GraphBuilder::new(TimeUnits::Generations);
        b.resolve().unwrap();
    }

    #[test]
    fn add_deme_with_epochs() {
        let mut b = GraphBuilder::new(TimeUnits::Generations);
        let edata = EpochData {
            start_size: Some(DemeSize::try_from(100.0).unwrap()),
            ..Default::default()
        };
        b.add_deme("CEU", vec![edata], DemeHistory::default(), None)
            .unwrap();
        let _graph = b.resolve().unwrap();
    }

    #[test]
    fn use_proportion_for_proportions() {
        let p = Proportion::try_from(0.5).unwrap();
        let _ = DemeHistory {
            proportions: Some(vec![p, p]),
            ..Default::default()
        };
    }

    #[test]
    fn builder_deme_defaults() {
        let defaults = DemeDefaults {
            epoch: EpochData {
                end_size: Some(DemeSize::try_from(100.).unwrap()),
                ..Default::default()
            },
        };
        let history = DemeHistory {
            defaults,
            ..Default::default()
        };
        let mut b = GraphBuilder::new(TimeUnits::Generations);
        b.add_deme("YRB", vec![], history, None).unwrap();
        b.resolve().unwrap();
    }
}
