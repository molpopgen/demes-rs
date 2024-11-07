use crate::DemesError;
use crate::Graph;
use crate::GraphBuilder;
use crate::InputGenerationTime;
use crate::InputProportion;
use crate::InputTime;
use crate::Time;
use crate::UnresolvedDemeHistory;
use crate::UnresolvedEpoch;
use crate::UnresolvedMigration;

// Remove all history from [when, infinity)
// NOTE: this function could take &Graph b/c it doesn't modify the input
// This function is a prototype for a future API to "slice" demographic models.
#[allow(dead_code)]
pub fn remove_since(graph: Graph, when: Time) -> Result<Graph, DemesError> {
    let generation_time = InputGenerationTime::from(f64::from(graph.generation_time()));

    let mut new_graph = GraphBuilder::new(graph.time_units(), Some(generation_time), None);

    let retained_deme_indexes = graph
        .demes
        .iter()
        .enumerate()
        .filter(|(_, deme)| deme.end_time() < when)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    let mut retained_deme_names = vec![];
    for i in retained_deme_indexes {
        let deme = graph.deme(i);
        let mut ancestors = vec![];
        let mut proportions: Vec<InputProportion> = vec![];
        for (name, proportion) in graph
            .deme(i)
            .ancestor_names()
            .iter()
            .zip(graph.deme(i).proportions().iter())
        {
            if retained_deme_names.contains(name) {
                ancestors.push(name.to_string());
                proportions.push(f64::from(*proportion).into());
            }
        }
        let mut history = UnresolvedDemeHistory::default();
        if !ancestors.is_empty() {
            history.start_time = Some(deme.start_time().into())
        }
        if !ancestors.is_empty() {
            history.ancestors = Some(ancestors);
            history.proportions = Some(proportions);
        }
        retained_deme_names.push(deme.name().to_string());
        let mut epochs: Vec<UnresolvedEpoch> = vec![];
        for e in deme.epochs().iter().filter(|e| e.end_time() < when) {
            let ue = UnresolvedEpoch {
                end_time: Some(e.end_time().into()),
                start_size: Some(f64::from(e.start_size()).into()),
                end_size: Some(f64::from(e.end_size()).into()),
                size_function: Some(e.size_function()),
                cloning_rate: Some(f64::from(e.cloning_rate()).into()),
                selfing_rate: Some(f64::from(e.selfing_rate()).into()),
            };
            epochs.push(ue)
        }
        new_graph.add_deme(
            deme.name(),
            epochs,
            history,
            if deme.description().is_empty() {
                None
            } else {
                Some(deme.description())
            },
        )
    }

    for m in graph.migrations().iter().filter(|m| {
        retained_deme_names.iter().any(|n| n == m.source())
            && retained_deme_names.iter().any(|n| n == m.dest())
            && m.end_time() < when
    }) {
        let mig = UnresolvedMigration {
            source: Some(m.source().to_string()),
            dest: Some(m.dest().to_string()),
            start_time: if m.start_time() > when {
                Some(when.into())
            } else {
                Some(m.start_time().into())
            },
            end_time: Some(m.end_time().into()),
            rate: Some(f64::from(m.rate()).into()),
            ..Default::default()
        };
        new_graph.add_migration(mig);
    }

    for pulse in graph.pulses().iter().filter(|p| {
        p.time() < when
            && retained_deme_names.iter().any(|n| n == p.dest())
            && p.sources()
                .iter()
                .all(|s| retained_deme_names.iter().any(|n| n == s))
    }) {
        let sources = pulse
            .sources()
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>();
        new_graph.add_pulse(
            Some(&sources),
            Some(pulse.dest()),
            Some(InputTime::from(pulse.time())),
            Some(pulse.proportions().iter().cloned().map(f64::from)),
        )
    }

    if let Some(metadata) = graph.metadata() {
        if let Err(e) = new_graph.set_toplevel_metadata(metadata.as_raw_ref()) {
            return Err(DemesError::GraphError(format!(
                "failed to set toplevel metadata: {e:?}"
            )));
        }
    }

    new_graph.resolve()
}

// Remove all history from [0, when)
// NOTE: this function could take &Graph b/c it doesn't modify the input
// This function is a prototype for a future API to "slice" demographic models.
#[allow(dead_code)]
pub fn remove_before(graph: Graph, when: Time) -> Result<Graph, DemesError> {
    let generation_time = InputGenerationTime::from(f64::from(graph.generation_time()));

    let mut new_graph = GraphBuilder::new(graph.time_units(), Some(generation_time), None);
    let retained_deme_indexes = graph
        .demes
        .iter()
        .enumerate()
        // NOTE: this is different from the other fn
        .filter(|(_, deme)| deme.start_time() > when)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();

    let mut retained_deme_names = vec![];
    for i in retained_deme_indexes {
        let deme = graph.deme(i);
        let mut ancestors = vec![];
        let mut proportions: Vec<InputProportion> = vec![];
        for (name, proportion) in graph
            .deme(i)
            .ancestor_names()
            .iter()
            .zip(graph.deme(i).proportions().iter())
        {
            if retained_deme_names.contains(name) {
                ancestors.push(name.to_string());
                proportions.push(f64::from(*proportion).into());
            }
        }
        let mut history = UnresolvedDemeHistory::default();
        if !ancestors.is_empty() {
            history.start_time = Some(deme.start_time().into())
        }
        if !ancestors.is_empty() {
            history.ancestors = Some(ancestors);
            history.proportions = Some(proportions);
        }
        retained_deme_names.push(deme.name().to_string());
        let mut epochs: Vec<UnresolvedEpoch> = vec![];
        for e in deme.epochs().iter().filter(|e| e.start_time() > when) {
            let ue = UnresolvedEpoch {
                // NOTE: this is a big diff from the remove_before fn!
                end_time: if e.end_time() < when {
                    Some(when.into())
                } else {
                    Some(e.end_time().into())
                },
                start_size: Some(f64::from(e.start_size()).into()),
                end_size: Some(f64::from(e.end_size()).into()),
                size_function: Some(e.size_function()),
                cloning_rate: Some(f64::from(e.cloning_rate()).into()),
                selfing_rate: Some(f64::from(e.selfing_rate()).into()),
            };
            epochs.push(ue)
        }
        new_graph.add_deme(
            deme.name(),
            epochs,
            history,
            if deme.description().is_empty() {
                None
            } else {
                Some(deme.description())
            },
        )
    }

    for m in graph.migrations().iter().filter(|m| {
        retained_deme_names.iter().any(|n| n == m.source())
            && retained_deme_names.iter().any(|n| n == m.dest())
            && m.start_time() >= when
    }) {
        let mig = UnresolvedMigration {
            source: Some(m.source().to_string()),
            dest: Some(m.dest().to_string()),
            start_time: Some(m.start_time().into()),
            end_time: if m.end_time() <= when {
                Some(when.into())
            } else {
                Some(m.end_time().into())
            },
            rate: Some(f64::from(m.rate()).into()),
            ..Default::default()
        };
        new_graph.add_migration(mig);
    }

    for pulse in graph.pulses().iter().filter(|p| {
        p.time() > when
            && retained_deme_names.iter().any(|n| n == p.dest())
            && p.sources()
                .iter()
                .all(|s| retained_deme_names.iter().any(|n| n == s))
    }) {
        let sources = pulse
            .sources()
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<_>>();
        new_graph.add_pulse(
            Some(&sources),
            Some(pulse.dest()),
            Some(InputTime::from(pulse.time())),
            Some(pulse.proportions().iter().cloned().map(f64::from)),
        )
    }

    if let Some(metadata) = graph.metadata() {
        if let Err(e) = new_graph.set_toplevel_metadata(metadata.as_raw_ref()) {
            return Err(DemesError::GraphError(format!(
                "failed to set toplevel metadata: {e:?}"
            )));
        }
    }

    new_graph.resolve()
}

#[cfg(test)]
mod test_remove_since {
    use super::remove_since;

    static SIMPLE_TWO_DEME_GRAPH: &str = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
";

    static SIMPLE_TWO_DEME_GRAPH_WITH_METADATA: &str = "
 time_units: generations
 metadata:
  x: 1
  y: 2
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
";

    static SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0: &str = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
 migrations:
  - demes: [ancestor1, ancestor2]
    rate: 0.25
    start_time: 100
    end_time: 45
";

    static SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0: &str = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
 pulses:
  - sources: [ancestor1]
    dest: ancestor2
    proportions: [0.33]
    time: 40
";

    #[test]
    fn test_simple_two_deme_graph_0() {
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        // This clipping will leave the graph unchanged.
        let clipped = remove_since(graph.clone(), 30.0.try_into().unwrap()).unwrap();
        assert_eq!(graph, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_migration_0() {
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0).unwrap();
        // This clipping will clip the migration interval.
        let clipped = remove_since(graph.clone(), 50.0.try_into().unwrap()).unwrap();
        let expected_yaml: &str = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
 migrations:
  - demes: [ancestor1, ancestor2]
    rate: 0.25
    start_time: 50
    end_time: 45
";
        let expected = crate::loads(expected_yaml).unwrap();
        assert_eq!(expected, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_migration_1() {
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0).unwrap();
        // This clipping will entirely remove migrations
        let clipped = remove_since(graph, 45.0.try_into().unwrap()).unwrap();
        let expected = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        assert_eq!(expected, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_pulse_0() {
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
        // This clipping will leave the graph unchanged.
        let clipped = remove_since(graph.clone(), 41.0.try_into().unwrap()).unwrap();
        assert_eq!(graph, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_pulse_1() {
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
        // This clipping will leave remove pulses from the graph.
        let clipped = remove_since(graph, 40.0.try_into().unwrap()).unwrap();
        let expected = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        assert_eq!(expected, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_1() {
        let expected_result = "
         time_units: generations
         demes:
          - name: derived
            epochs:
             - start_size: 50
        ";
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        let expected_result = crate::loads(expected_result).unwrap();

        // This clipping will leave the graph with a single population
        // that has no ancestors, no proportions, and start time of infinity
        let clipped = remove_since(graph.clone(), 20.0.try_into().unwrap()).unwrap();
        assert_eq!(expected_result, clipped);

        // This clipping will leave the graph with a single population
        // that has no ancestors, no proportions, and start time of infinity
        let clipped = remove_since(graph, 1.0.try_into().unwrap()).unwrap();
        assert_eq!(expected_result, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_metadata() {
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_METADATA).unwrap();

        // Leaves graph unchanged
        let clipped = remove_since(graph.clone(), 30.0.try_into().unwrap()).unwrap();
        assert_eq!(graph, clipped);
    }
}

#[cfg(test)]
mod test_remove_before {
    use super::remove_before;

    static SIMPLE_TWO_DEME_GRAPH: &str = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
";

    static SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0: &str = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
 migrations:
  - demes: [ancestor1, ancestor2]
    rate: 0.25
    start_time: 100
    end_time: 45
";

    static SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0: &str = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
 pulses:
  - sources: [ancestor1]
    dest: ancestor2
    proportions: [0.33]
    time: 40
";

    #[test]
    fn test_simple_two_deme_graph() {
        let expected = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 20
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 20
  - name: derived
    ancestors: [ancestor1, ancestor2]
    proportions: [0.75, 0.25]
    start_time: 20
    epochs:
     - start_size: 50
       end_time: 10
";
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        let expected_graph = crate::loads(expected).unwrap();
        let clipped = remove_before(graph, 10.0.try_into().unwrap()).unwrap();
        assert_eq!(clipped, expected_graph);
    }

    #[test]
    fn test_simple_two_deme_graph_1() {
        let expected = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 50
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 50
";
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        let expected_graph = crate::loads(expected).unwrap();
        let clipped = remove_before(graph, 50.0.try_into().unwrap()).unwrap();
        assert_eq!(clipped, expected_graph);
    }

    #[test]
    fn test_simple_two_deme_graph_with_migration_0() {
        let expected = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 50
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 50
 migrations:
  - demes: [ancestor1, ancestor2]
    rate: 0.25
    start_time: 100
    end_time: 50
";
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0).unwrap();
        let expected_graph = crate::loads(expected).unwrap();
        let clipped = remove_before(graph, 50.0.try_into().unwrap()).unwrap();
        assert_eq!(clipped, expected_graph);
    }

    #[test]
    fn test_simple_two_deme_graph_with_pulse_0() {
        let expected = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 40
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 40
";
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
        let expected_graph = crate::loads(expected).unwrap();
        let clipped = remove_before(graph, 40.0.try_into().unwrap()).unwrap();
        assert_eq!(clipped, expected_graph);
    }

    #[test]
    fn test_simple_two_deme_graph_with_pulse_1() {
        let expected = "
 time_units: generations
 demes:
  - name: ancestor1
    epochs:
     - start_size: 50
       end_time: 39
  - name: ancestor2
    epochs:
     - start_size: 50
       end_time: 39
 pulses:
  - sources: [ancestor1]
    dest: ancestor2
    proportions: [0.33]
    time: 40
";
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
        let expected_graph = crate::loads(expected).unwrap();
        let clipped = remove_before(graph, 39.0.try_into().unwrap()).unwrap();
        assert_eq!(clipped, expected_graph);
    }
}
