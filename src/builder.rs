use crate::specification::Deme;
use crate::specification::DemeHistory;
use crate::specification::EpochData;
use crate::specification::GenerationTime;
use crate::specification::Graph;
use crate::specification::GraphDefaults;
use crate::specification::MigrationRate;
use crate::specification::Proportion;
use crate::specification::Time;
use crate::specification::TimeUnits;
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
        let mut b = GraphBuilder::new_generations(None);
        b.add_deme("YRB", vec![], history, None).unwrap();
        b.resolve().unwrap();
    }
}

#[cfg(test)]
mod test_toplevel_defaults {
    use crate::specification::{DemeSize, Pulse, TopLevelDemeDefaults, UnresolvedMigration};

    use super::*;

    #[test]
    fn builder_toplevel_pulse_defaults() {
        let yaml = "
time_units: years
generation_time: 25
defaults:
  pulse: {sources: [A], dest: B, proportions: [0.25], time: 100}
demes:
  - name: A
    epochs: 
     - start_size: 100
  - name: B
    epochs:
     - start_size: 250 
";
        let graph_from_yaml = crate::loads(yaml).unwrap();

        let toplevel_defaults = GraphDefaults {
            pulse: Pulse {
                sources: Some(vec!["A".to_string()]),
                dest: Some("B".to_string()),
                proportions: Some(vec![Proportion::try_from(0.25).unwrap()]),
                time: Some(Time::try_from(100.).unwrap()),
            },
            ..Default::default()
        };

        let epochs_a = EpochData {
            start_size: Some(DemeSize::try_from(100.0).unwrap()),
            ..Default::default()
        };
        let epochs_b = EpochData {
            start_size: Some(DemeSize::try_from(250.0).unwrap()),
            ..Default::default()
        };

        let mut builder = GraphBuilder::new(
            TimeUnits::Years,
            Some(GenerationTime::from(25.0)),
            Some(toplevel_defaults),
        );
        builder
            .add_deme("A", vec![epochs_a], DemeHistory::default(), None)
            .unwrap();
        builder
            .add_deme("B", vec![epochs_b], DemeHistory::default(), None)
            .unwrap();
        let graph_from_builder = builder.resolve().unwrap();
        assert_eq!(graph_from_yaml, graph_from_builder);
    }

    #[test]
    fn builder_toplevel_epoch_defaults() {
        let _ = GraphDefaults {
            epoch: EpochData {
                end_time: Some(Time::try_from(100.0).unwrap()),
                ..Default::default()
            },
            ..Default::default()
        };
    }

    #[test]
    fn builder_toplevel_migration_defaults() {
        let _ = GraphDefaults {
            migration: UnresolvedMigration {
                source: Some("A".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
    }

    #[test]
    fn builder_toplevel_deme_defaults() {
        {
            let _ = GraphDefaults {
                deme: TopLevelDemeDefaults {
                    description: Some("bananas".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            };
        }

        {
            let _ = GraphDefaults {
                deme: TopLevelDemeDefaults {
                    start_time: Some(Time::try_from(100.0).unwrap()),
                    ..Default::default()
                },
                ..Default::default()
            };
        }

        {
            let _ = GraphDefaults {
                deme: TopLevelDemeDefaults {
                    ancestors: Some(vec!["A".to_string()]),
                    ..Default::default()
                },
                ..Default::default()
            };
        }

        {
            let _ = GraphDefaults {
                deme: TopLevelDemeDefaults {
                    proportions: Some(vec![Proportion::try_from(1.0).unwrap()]),
                    ..Default::default()
                },
                ..Default::default()
            };
        }
    }
}
