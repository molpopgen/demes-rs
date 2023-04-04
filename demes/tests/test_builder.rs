use demes::GraphBuilder;
use demes::GraphDefaults;
use demes::InputDemeSize;
use demes::InputGenerationTime;
use demes::InputProportion;
use demes::InputTime;
use demes::TimeUnits;
use demes::TopLevelDemeDefaults;
use demes::UnresolvedDemeHistory;
use demes::UnresolvedEpoch;
use demes::UnresolvedMigration;
use demes::UnresolvedPulse;

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
    let graph_from_yaml = demes::loads(yaml).unwrap();

    let toplevel_defaults = GraphDefaults {
        pulse: UnresolvedPulse {
            sources: Some(vec!["A".to_string()]),
            dest: Some("B".to_string()),
            proportions: Some(vec![InputProportion::from(0.25)]),
            time: Some(InputTime::from(100.)),
        },
        ..Default::default()
    };

    let epochs_a = UnresolvedEpoch {
        start_size: Some(InputDemeSize::from(100.0)),
        ..Default::default()
    };
    let epochs_b = UnresolvedEpoch {
        start_size: Some(InputDemeSize::from(250.0)),
        ..Default::default()
    };

    let mut builder = GraphBuilder::new(
        TimeUnits::Years,
        Some(InputGenerationTime::from(25.0)),
        Some(toplevel_defaults),
    );
    builder.add_deme("A", vec![epochs_a], UnresolvedDemeHistory::default(), None);
    builder.add_deme("B", vec![epochs_b], UnresolvedDemeHistory::default(), None);
    let graph_from_builder = builder.resolve().unwrap();
    assert_eq!(graph_from_yaml, graph_from_builder);
}

#[test]
fn builder_toplevel_epoch_defaults() {
    let _ = GraphDefaults {
        epoch: UnresolvedEpoch {
            end_time: Some(InputTime::from(100.0)),
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
                start_time: Some(InputTime::from(100.0)),
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
                proportions: Some(vec![InputProportion::from(1.0)]),
                ..Default::default()
            },
            ..Default::default()
        };
    }
}

#[test]
fn test_metadata_round_trip_through_builder() {
    #[derive(serde::Serialize, Debug, serde::Deserialize, Eq, PartialEq)]
    struct MyMetaData {
        foo: i32,
        bar: String,
    }
    let edata = demes::UnresolvedEpoch {
        start_size: Some(demes::InputDemeSize::from(100.0)),
        ..Default::default()
    };
    let mut builder = demes::GraphBuilder::new_generations(None);
    builder.add_deme(
        "CEU",
        vec![edata],
        demes::UnresolvedDemeHistory::default(),
        None,
    );
    let md = MyMetaData {
        foo: 3,
        bar: "string".to_owned(),
    };
    builder.set_toplevel_metadata(&md).unwrap();
    let graph = builder.resolve().unwrap();
    let metadata = graph.metadata().unwrap();
    let x: MyMetaData = serde_yaml::from_str(&metadata.as_yaml_string().unwrap()).unwrap();
    assert_eq!(x, md);
}
