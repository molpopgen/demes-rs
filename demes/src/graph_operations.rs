use crate::AsymmetricMigration;
use crate::Deme;
use crate::DemesError;
use crate::Epoch;
use crate::Graph;
use crate::GraphBuilder;
use crate::InputGenerationTime;
use crate::InputProportion;
use crate::InputTime;
use crate::Pulse;
use crate::Time;
use crate::UnresolvedDemeHistory;
use crate::UnresolvedEpoch;
use crate::UnresolvedMigration;

fn retained_deme_indexes<C>(graph: &Graph, criterion: C) -> Vec<usize>
where
    C: Fn(&Deme) -> bool,
{
    graph
        .demes
        .iter()
        .enumerate()
        .filter(|(_, deme)| criterion(deme))
        .map(|(index, _)| index)
        .collect::<Vec<_>>()
}

struct Callbacks<
    D: Fn(&Deme) -> bool,
    E: Fn(&Epoch) -> bool,
    M: Fn(&AsymmetricMigration) -> bool,
    P: Fn(&Pulse) -> bool,
    EE: Fn(Time) -> Option<InputTime>,
    MS: Fn(Time) -> Option<InputTime>,
    ME: Fn(Time) -> Option<InputTime>,
> {
    keep_deme: D,
    keep_epoch: E,
    keep_migration: M,
    keep_pulse: P,
    epoch_end_time: EE,
    migration_start_time: MS,
    migration_end_time: ME,
}

fn liftover_demes<K, E>(
    graph: &Graph,
    retained_deme_indexes: Vec<usize>,
    retained_deme_names: &[String],
    keep_epoch: K,
    epoch_end_time: E,
    new_graph: &mut GraphBuilder,
) where
    K: Fn(&Epoch) -> bool,
    E: Fn(Time) -> Option<InputTime>,
{
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
        let mut epochs: Vec<UnresolvedEpoch> = vec![];
        for e in deme.epochs().iter().filter(|e| keep_epoch(e)) {
            let ue = UnresolvedEpoch {
                end_time: epoch_end_time(e.end_time()),
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
}

fn liftover_migrations<K, S, E>(
    graph: &Graph,
    retained_deme_names: &[String],
    keep_migration: K,
    make_start_time: S,
    make_end_time: E,
    new_graph: &mut GraphBuilder,
) where
    K: Fn(&AsymmetricMigration) -> bool,
    S: Fn(Time) -> Option<InputTime>,
    E: Fn(Time) -> Option<InputTime>,
{
    for m in graph.migrations().iter().filter(|&m| {
        retained_deme_names.iter().any(|n| n == m.source())
            && retained_deme_names.iter().any(|n| n == m.dest())
            && keep_migration(m)
    }) {
        let mig = UnresolvedMigration {
            source: Some(m.source().to_string()),
            dest: Some(m.dest().to_string()),
            start_time: make_start_time(m.start_time()),
            end_time: make_end_time(m.end_time()),
            rate: Some(f64::from(m.rate()).into()),
            ..Default::default()
        };
        new_graph.add_migration(mig);
    }
}

fn liftover_pulses<F>(
    graph: &Graph,
    retained_deme_names: &[String],
    callback: F,
    new_graph: &mut GraphBuilder,
) where
    F: Fn(&Pulse) -> bool,
{
    for pulse in graph.pulses().iter().filter(|&p| {
        callback(p)
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
}

fn liftover_metadata(graph: &Graph, new_graph: &mut GraphBuilder) -> Result<(), DemesError> {
    if let Some(metadata) = graph.metadata() {
        if let Err(e) = new_graph.set_toplevel_metadata(metadata.as_raw_ref()) {
            return Err(DemesError::GraphError(format!(
                "failed to set toplevel metadata: {e:?}"
            )));
        }
    }
    Ok(())
}

fn remove_history<
    D: Fn(&Deme) -> bool,
    E: Fn(&Epoch) -> bool,
    M: Fn(&AsymmetricMigration) -> bool,
    P: Fn(&Pulse) -> bool,
    EE: Fn(Time) -> Option<InputTime>,
    MS: Fn(Time) -> Option<InputTime>,
    ME: Fn(Time) -> Option<InputTime>,
>(
    graph: Graph,
    callbacks: Callbacks<D, E, M, P, EE, MS, ME>,
) -> Result<Graph, DemesError> {
    let generation_time = InputGenerationTime::from(f64::from(graph.generation_time()));
    let mut new_graph = GraphBuilder::new(graph.time_units(), Some(generation_time), None);
    let retained_deme_indexes = retained_deme_indexes(&graph, callbacks.keep_deme);
    let retained_deme_names = retained_deme_indexes
        .iter()
        .cloned()
        .map(|index| graph.deme(index).name().to_string())
        .collect::<Vec<_>>();
    liftover_demes(
        &graph,
        retained_deme_indexes,
        &retained_deme_names,
        callbacks.keep_epoch,
        callbacks.epoch_end_time,
        &mut new_graph,
    );
    liftover_migrations(
        &graph,
        &retained_deme_names,
        callbacks.keep_migration,
        callbacks.migration_start_time,
        callbacks.migration_end_time,
        &mut new_graph,
    );
    liftover_pulses(
        &graph,
        &retained_deme_names,
        callbacks.keep_pulse,
        &mut new_graph,
    );
    liftover_metadata(&graph, &mut new_graph)?;
    new_graph.resolve()
}

// Remove all history from [when, infinity)
// NOTE: this function could take &Graph b/c it doesn't modify the input
// This function is a prototype for a future API to "slice" demographic models.
#[allow(dead_code)]
pub fn remove_since(graph: Graph, when: Time) -> Result<Graph, DemesError> {
    let callbacks = Callbacks {
        keep_deme: |d: &Deme| d.end_time() < when,
        keep_epoch: |e: &Epoch| e.end_time() < when,
        keep_migration: |m: &AsymmetricMigration| m.end_time() < when,
        keep_pulse: |m: &Pulse| m.time() < when,
        epoch_end_time: |t: Time| Some(t.into()),
        migration_start_time: |t: Time| {
            if t > when {
                Some(when.into())
            } else {
                Some(t.into())
            }
        },
        migration_end_time: |t: Time| Some(t.into()),
    };

    remove_history(graph, callbacks)
}

// Remove all history from [0, when), leaving a history from [when, ....)
// NOTE: this function could take &Graph b/c it doesn't modify the input
// This function is a prototype for a future API to "slice" demographic models.
#[allow(dead_code)]
pub fn remove_before(graph: Graph, when: Time) -> Result<Graph, DemesError> {
    let callbacks = Callbacks {
        keep_deme: |d: &Deme| d.start_time() > when,
        keep_epoch: |e: &Epoch| e.start_time() > when,
        keep_migration: |m: &AsymmetricMigration| m.start_time() > when,
        keep_pulse: |m: &Pulse| m.time() > when,
        epoch_end_time: |t: Time| {
            if t < when {
                Some(when.into())
            } else {
                Some(t.into())
            }
        },
        migration_start_time: |t: Time| Some(t.into()),
        migration_end_time: |t: Time| {
            if t <= when {
                Some(when.into())
            } else {
                Some(t.into())
            }
        },
    };

    remove_history(graph, callbacks)
}

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
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

#[cfg(test)]
mod test_remove_since {
    use super::remove_since;

    #[test]
    fn test_simple_two_deme_graph_0() {
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH).unwrap();
        // This clipping will leave the graph unchanged.
        let clipped = remove_since(graph.clone(), 30.0.try_into().unwrap()).unwrap();
        assert_eq!(graph, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_migration_0() {
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0).unwrap();
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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0).unwrap();
        // This clipping will entirely remove migrations
        let clipped = remove_since(graph, 45.0.try_into().unwrap()).unwrap();
        let expected = crate::loads(super::SIMPLE_TWO_DEME_GRAPH).unwrap();
        assert_eq!(expected, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_pulse_0() {
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
        // This clipping will leave the graph unchanged.
        let clipped = remove_since(graph.clone(), 41.0.try_into().unwrap()).unwrap();
        assert_eq!(graph, clipped);
    }

    #[test]
    fn test_simple_two_deme_graph_with_pulse_1() {
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
        // This clipping will leave remove pulses from the graph.
        let clipped = remove_since(graph, 40.0.try_into().unwrap()).unwrap();
        let expected = crate::loads(super::SIMPLE_TWO_DEME_GRAPH).unwrap();
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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH).unwrap();
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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_METADATA).unwrap();

        // Leaves graph unchanged
        let clipped = remove_since(graph.clone(), 30.0.try_into().unwrap()).unwrap();
        assert_eq!(graph, clipped);
    }
}

#[cfg(test)]
mod test_remove_before {
    use super::remove_before;

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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH).unwrap();
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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH).unwrap();
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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_MIGRATION_0).unwrap();
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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
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
        let graph = crate::loads(super::SIMPLE_TWO_DEME_GRAPH_WITH_PULSE_0).unwrap();
        let expected_graph = crate::loads(expected).unwrap();
        let clipped = remove_before(graph, 39.0.try_into().unwrap()).unwrap();
        assert_eq!(clipped, expected_graph);
    }
}

#[test]
fn slice_to_empty() {
    {
        // This graph goes from [0, inf)
        // We first slice it down to [40, inf) and then from [min, 40).
        // The last interval is not inclusive of the result of the first slice,
        // leaving an empty graph, which is an error.
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        let graph = remove_before(graph, 40.0.try_into().unwrap()).unwrap();
        assert!(remove_since(graph, 40.0.try_into().unwrap()).is_err());
    }
    // Reverse the order of operations
    {
        let graph = crate::loads(SIMPLE_TWO_DEME_GRAPH).unwrap();
        // Slice from [0, inf) to [0, inf), but removing all events after 40
        let graph = remove_since(graph, 40.0.try_into().unwrap()).unwrap();
        println!("{graph:?}");
        // Then slice down to [40, inf), which is still valid
        let clipped = remove_before(graph.clone(), 40.0.try_into().unwrap()).unwrap();
        println!("{clipped:?}");
        assert!(remove_before(graph, 40.0.try_into().unwrap()).is_ok());
    }
}
