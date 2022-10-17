use demes::{AsymmetricMigration, MigrationRate, SizeFunction, Time};

#[derive(Eq, PartialEq)]
struct ExpectedMigration {
    source: String,
    dest: String,
    rate: MigrationRate,
    start_time: Time,
    end_time: Time,
}

impl ExpectedMigration {
    fn new<N: ToString, M: Into<MigrationRate>, S: Into<Time>, E: Into<Time>>(
        source: N,
        dest: N,
        rate: M,
        start_time: S,
        end_time: E,
    ) -> Result<Self, demes::DemesError> {
        let rate = rate.into();
        let start_time = start_time.into();
        let end_time = end_time.into();
        Ok(Self {
            source: source.to_string(),
            dest: dest.to_string(),
            rate,
            start_time,
            end_time,
        })
    }
}

impl From<AsymmetricMigration> for ExpectedMigration {
    fn from(value: AsymmetricMigration) -> Self {
        Self {
            source: value.source().to_string(),
            dest: value.dest().to_string(),
            rate: value.rate(),
            start_time: value.start_time(),
            end_time: value.end_time(),
        }
    }
}

fn test_graph_equality_after_round_trip(
    graph: &demes::Graph,
) -> Result<bool, Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(graph)?;
    let round_trip = demes::loads(&yaml)?;
    Ok(*graph == round_trip)
}

macro_rules! assert_graph_equality_after_round_trip {
    ($graph: ident) => {
        match test_graph_equality_after_round_trip(&$graph) {
            Ok(b) => assert!(b),
            Err(e) => panic!("{}", e.to_string()),
        }
    };
}

#[test]
fn test_tutorial_example_01() {
    let yaml = "
# Comments start with a hash.
description:
  Asymmetric migration between two extant demes.
time_units: generations
defaults:
  epoch:
    start_size: 5000
demes:
  - name: X
    epochs:
      - end_time: 1000
  - name: A
    ancestors: [X]
  - name: B
    ancestors: [X]
    epochs:
      - start_size: 2000
        end_time: 500
      - start_size: 400
        end_size: 10000
migrations:
  - source: A
    dest: B
    rate: 1e-4
";
    let g = demes::loads(yaml).unwrap();
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn tutorial_example_03() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
";
    let g = demes::loads(yaml).unwrap();
    assert_eq!(g.num_demes(), 1);
    assert_eq!(
        f64::from(g.get_deme_from_name("A").unwrap().start_time()),
        f64::INFINITY,
    );
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn replacement_with_size_change() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
        end_time: 1000
  - name: B
    ancestors: [A]
    epochs:
      - start_size: 2000
";
    let g = demes::loads(yaml).unwrap();
    assert_eq!(g.num_demes(), 2);
    assert_eq!(g.demes().len(), 2);

    for d in g.demes() {
        if *d.name() == "A" {
            assert_eq!(d.num_ancestors(), 0);
        } else {
            assert_eq!(d.num_ancestors(), 1);

            // iterate over ancestor HashMap of {ancestor name => ancestor Deme}
            for (name, deme) in d.ancestors().iter() {
                assert_eq!(name, "A");
                assert_eq!(*deme.name(), *name);
                assert_eq!(deme.num_ancestors(), 0);
            }

            // Iterate over just the names
            assert!(d.ancestors().keys().all(|ancestor| *ancestor == "A"));

            // Iterate ignoring the names
            assert!(d.ancestors().values().all(|deme| *deme.name() == "A"));

            // With only 1 ancestor, there is exactly 1 proportion
            // represeting 100% ancestry
            assert_eq!(d.proportions().len(), 1);
            assert_eq!(f64::from(d.proportions()[0]), 1.0);
        }
    }
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn default_epoch_sizes() {
    let yaml = "
time_units: generations
defaults:
  epoch:
    start_size: 1000
demes:
  - name: A
";
    let g = demes::loads(yaml).unwrap();
    assert_eq!(g.num_demes(), 1);

    // .deme(at) returns a strong reference,
    // thus increasing the reference count
    assert_eq!(*g.deme(0).name(), "A");
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn tutorial_example_21() {
    let yaml = "
description: The Gutenkunst et al. (2009) OOA model.
doi:
- https://doi.org/10.1371/journal.pgen.1000695
time_units: years
generation_time: 25

demes:
- name: ancestral
  description: Equilibrium/root population
  epochs:
  - {end_time: 220e3, start_size: 7300}
- name: AMH
  description: Anatomically modern humans
  ancestors: [ancestral]
  epochs:
  - {end_time: 140e3, start_size: 12300}
- name: OOA
  description: Bottleneck out-of-Africa population
  ancestors: [AMH]
  epochs:
  - {end_time: 21.2e3, start_size: 2100}
- name: YRI
  description: Yoruba in Ibadan, Nigeria
  ancestors: [AMH]
  epochs:
  - start_size: 12300
- name: CEU
  description: Utah Residents (CEPH) with Northern and Western European Ancestry
  ancestors: [OOA]
  epochs:
  - {start_size: 1000, end_size: 29725}
- name: CHB
  description: Han Chinese in Beijing, China
  ancestors: [OOA]
  epochs:
  - {start_size: 510, end_size: 54090}

migrations:
- {demes: [YRI, OOA], rate: 25e-5}
- {demes: [YRI, CEU], rate: 3e-5}
- {demes: [YRI, CHB], rate: 1.9e-5}
- {demes: [CEU, CHB], rate: 9.6e-5}
";

    let g = demes::loads(yaml).unwrap();
    let generation_time: f64 = g.generation_time().unwrap().into();
    assert_eq!(generation_time, 25.0);
    assert!(matches!(g.time_units(), demes::TimeUnits::Years,));
    assert_eq!(g.time_units().to_string(), "years".to_string());
    assert_eq!(g.migrations().len(), 8);

    let expected_resolved_migrations = vec![
        ExpectedMigration::new("YRI", "OOA", 25e-5, 140e3, 21.2e3).unwrap(),
        ExpectedMigration::new("OOA", "YRI", 25e-5, 140e3, 21.2e3).unwrap(),
        ExpectedMigration::new("YRI", "CEU", 3e-5, 21.2e3, 0.0).unwrap(),
        ExpectedMigration::new("CEU", "YRI", 3e-5, 21.2e3, 0.0).unwrap(),
        ExpectedMigration::new("YRI", "CHB", 1.9e-5, 21.2e3, 0.0).unwrap(),
        ExpectedMigration::new("CHB", "YRI", 1.9e-5, 21.2e3, 0.0).unwrap(),
        ExpectedMigration::new("CEU", "CHB", 9.6e-5, 21.2e3, 0.0).unwrap(),
        ExpectedMigration::new("CHB", "CEU", 9.6e-5, 21.2e3, 0.0).unwrap(),
    ];

    assert!(g
        .migrations()
        .iter()
        .all(|m| expected_resolved_migrations.contains(&ExpectedMigration::from(m.clone()))));

    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn test_size_propagation() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
        end_time: 1000
  - name: B
    epochs:
      - start_size: 2000
  - name: C
    ancestors: [A, B]
    proportions: [0.5, 0.5]
    start_time: 1000
    epochs:
      - start_size: 503
";
    let g = demes::loads(yaml).unwrap();
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn test_start_time_handling() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    ancestors: [A]
    start_time: 100
    epochs:
      - start_size: 1000
";
    let g = demes::loads(yaml).unwrap();
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn tutorial_example_17() {
    let yaml = "
time_units: generations
demes:
  - name: X
    epochs:
      - end_time: 1000
        start_size: 2000
  - name: A
    ancestors: [X]
    epochs:
      - start_size: 2000
  - name: B
    ancestors: [X]
    epochs:
      - start_size: 2000
pulses:
  - sources: [A]
    dest: B
    proportions: [0.05]
    time: 500
";
    let g = demes::loads(yaml).unwrap();
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn jacobs_el_al_2019_from_gallery() {
    let yaml = "
description: |
  A ten population model of out-of-Africa, including two pulses of
  Denisovan admixture into Papuans, and several pulses of Neandertal
  admixture into non-Africans.
  Most parameters are from Jacobs et al. (2019), Table S5 and Figure S5.
  This model is an extension of one from Malaspinas et al. (2016), thus
  some parameters are inherited from there.
time_units: generations
doi:
- https://doi.org/10.1016/j.cell.2019.02.035
- https://doi.org/10.1038/nature18299

demes:
- name: YRI
  epochs:
  - {end_time: 20225.0, start_size: 32671.0}
  - {end_time: 2218.0, start_size: 41563.0}
  - {end_time: 0, start_size: 48433.0}
- name: DenA
  ancestors: [YRI]
  start_time: 20225.0
  epochs:
  - {end_time: 15090.0, start_size: 13249.0}
  - {end_time: 12500.0, start_size: 100.0}
  - {end_time: 9750.0, start_size: 100.0}
  - {end_time: 0, start_size: 5083.0}
- name: NeaA
  ancestors: [DenA]
  start_time: 15090.0
  epochs:
  - {end_time: 3375.0, start_size: 13249.0}
  - {end_time: 0, start_size: 826.0}
- name: Den2
  ancestors: [DenA]
  start_time: 12500.0
  epochs:
  - start_size: 13249.0
- name: Den1
  ancestors: [DenA]
  start_time: 9750.0
  epochs:
  - start_size: 13249.0
- name: Nea1
  ancestors: [NeaA]
  start_time: 3375.0
  epochs:
  - start_size: 13249.0
- name: Ghost
  ancestors: [YRI]
  start_time: 2218.0
  epochs:
  - {end_time: 2119.0, start_size: 1394.0}
  - {end_time: 0, start_size: 8516.0}
- name: Papuan
  ancestors: [Ghost]
  start_time: 1784.0
  epochs:
  - {end_time: 1685.0, start_size: 243.0}
  - {end_time: 0, start_size: 8834.0}
- name: CHB
  ancestors: [Ghost]
  start_time: 1758.0
  epochs:
  - {end_time: 1659.0, start_size: 2231.0}
  - {end_time: 1293.0, start_size: 12971.0}
  - {end_time: 0, start_size: 9025.0}
- name: CEU
  ancestors: [CHB]
  start_time: 1293.0
  epochs:
  - start_size: 6962.0

migrations:
- {demes: [YRI, Ghost], rate: 0.000179, start_time: 1659.0}
- {demes: [CHB, Papuan], rate: 0.000572, start_time: 1659.0, end_time: 1293.0}
- {demes: [CHB, Papuan], rate: 5.72e-05, start_time: 1293.0}
- {demes: [CHB, Ghost], rate: 0.000442, start_time: 1659.0, end_time: 1293.0}
- {demes: [CEU, CHB], rate: 3.14e-05}
- {demes: [CEU, Ghost], rate: 0.000442}

pulses:
- {sources: [Nea1], dest: Ghost, time: 1853.0, proportions: [0.024]}
- {sources: [Den2], dest: Papuan, time: 1575.8620689655172, proportions: [0.018]}
- {sources: [Nea1], dest: CHB, time: 1566.0, proportions: [0.011]}
- {sources: [Nea1], dest: Papuan, time: 1412.0, proportions: [0.002]}
- {sources: [Den1], dest: Papuan, time: 1027.5862068965516, proportions: [0.022]}
- {sources: [Nea1], dest: CHB, time: 883.0, proportions: [0.002]}
";

    let g = demes::loads(yaml).unwrap();
    assert_graph_equality_after_round_trip!(g);
}

#[test]
fn test_pulses_are_stably_sorted() {
    let yaml1 = "
time_units: generations
demes:
  - name: X
    epochs:
      - end_time: 1000
        start_size: 2000
  - name: A
    ancestors: [X]
    epochs:
      - start_size: 2000
  - name: B
    ancestors: [X]
    epochs:
      - start_size: 2000
pulses:
  - sources: [A]
    dest: B
    proportions: [0.05]
    time: 500
  - sources: [A]
    dest: B
    proportions: [0.05]
    time: 501
  - sources: [B]
    dest: A
    proportions: [0.05]
    time: 501
";

    let yaml2 = "
time_units: generations
description: the pulses at 501 are in a different order
demes:
  - name: X
    epochs:
      - end_time: 1000
        start_size: 2000
  - name: A
    ancestors: [X]
    epochs:
      - start_size: 2000
  - name: B
    ancestors: [X]
    epochs:
      - start_size: 2000
pulses:
  - sources: [A]
    dest: B
    proportions: [0.05]
    time: 500
  - sources: [B]
    dest: A
    proportions: [0.05]
    time: 501
  - sources: [A]
    dest: B
    proportions: [0.05]
    time: 501
";

    let g = demes::loads(yaml1).unwrap();
    let expected_pulse_times = vec![
        demes::Time::try_from(501.).unwrap(),
        demes::Time::try_from(501.).unwrap(),
        demes::Time::try_from(500.).unwrap(),
    ];
    let pulse_times = g
        .pulses()
        .iter()
        .map(|pulse| pulse.time())
        .collect::<Vec<Time>>();
    assert_eq!(pulse_times, expected_pulse_times);
    assert_eq!(g.pulses()[0].sources(), &["A".to_string()]);
    assert_eq!(g.pulses()[0].dest(), "B");
    assert_eq!(g.pulses()[1].sources(), &["B".to_string()]);
    assert_eq!(g.pulses()[1].dest(), "A");
    assert_graph_equality_after_round_trip!(g);

    let g2 = demes::loads(yaml2).unwrap();
    let pulse_times = g
        .pulses()
        .iter()
        .map(|pulse| pulse.time())
        .collect::<Vec<Time>>();
    assert_eq!(pulse_times, expected_pulse_times);
    assert_eq!(g2.pulses()[0].sources(), &["B".to_string()]);
    assert_eq!(g2.pulses()[0].dest(), "A");
    assert_eq!(g2.pulses()[1].sources(), &["A".to_string()]);
    assert_eq!(g2.pulses()[1].dest(), "B");

    // The two graphs are not equal b/c the pulses
    // are sorted stable w.r.to time.
    assert_ne!(g, g2);
}

#[test]
fn linear_size_function_default() {
    let yaml = "
time_units: generations
defaults:
  epoch:
    start_size: 5000
    size_function: linear
demes:
  - name: X
    epochs:
      - end_time: 1000
      - end_size: 100

";
    let graph = demes::loads(yaml).unwrap();
    let sf = graph.deme(0).epochs()[0].size_function();
    assert!(matches!(sf, demes::SizeFunction::Constant));
    let sf = graph.deme(0).epochs()[1].size_function();
    assert!(matches!(sf, demes::SizeFunction::Linear));
}

#[test]
fn selfing_rate_default() {
    let yaml = "
time_units: generations
defaults:
  epoch:
    selfing_rate: 0.25
demes:
  - name: X
    epochs:
     - start_size: 5000
";
    let graph = demes::loads(yaml).unwrap();
    let selfing_rate = graph.deme(0).epochs()[0].selfing_rate();
    assert_eq!(0.25, f64::from(selfing_rate));
}

#[test]
fn cloning_rate_default() {
    let yaml = "
time_units: generations
defaults:
  epoch:
    cloning_rate: 0.25
demes:
  - name: X
    epochs:
     - start_size: 5000
";
    let graph = demes::loads(yaml).unwrap();
    let cloning_rate = graph.deme(0).epochs()[0].cloning_rate();
    assert_eq!(0.25, f64::from(cloning_rate));
}

#[test]
fn end_time_default() {
    let yaml = "
time_units: generations
defaults:
  epoch:
    end_time: 100
demes:
  - name: X
    epochs:
     - start_size: 5000
";
    let graph = demes::loads(yaml).unwrap();
    let end_time = graph.deme(0).epochs()[0].end_time();
    assert_eq!(100.0, f64::from(end_time));
}

// from demes-spec/test-cases/valid
#[test]
fn defaults_deme_many_epochs_local() {
    let yaml = "
description: Set epoch defaults using deme-local values.
time_units: generations

demes:
- name: deme0
  defaults:
    epoch: {cloning_rate: 0.5, end_size: 2, selfing_rate: 0.1, start_size: 1}
  epochs:
  - {end_time: 100, start_size: 1, end_size: 1}
  - {end_time: 3}
  - {end_time: 2}
  - {end_time: 1}
  - {end_time: 0}
";
    let g = demes::loads(yaml).unwrap();

    for deme in g.demes().iter() {
        assert_eq!(f64::from(deme.start_size()), 1.0);
    }

    for deme in g.demes().iter() {
        for epoch in deme.epochs().iter() {
            assert_eq!(f64::from(epoch.cloning_rate()), 0.5);
            assert_eq!(f64::from(epoch.selfing_rate()), 0.1);
        }
    }
    let expected_start_sizes = vec![1.; 5];
    let start_sizes = g
        .deme(0)
        .start_sizes()
        .iter()
        .map(|size| f64::from(*size))
        .collect::<Vec<f64>>();
    assert_eq!(start_sizes, expected_start_sizes);

    let expected_end_sizes = vec![1.0, 2.0, 2.0, 2.0, 2.0];
    let end_sizes = g.deme(0).end_sizes();
    let end_sizes = end_sizes
        .iter()
        .map(|size| f64::from(*size))
        .collect::<Vec<f64>>();
    assert_eq!(end_sizes, expected_end_sizes);
    let expected_start_times = vec![f64::INFINITY, 100., 3., 2., 1.];
    let start_times = g
        .deme(0)
        .start_times()
        .iter()
        .map(|time| f64::from(*time))
        .collect::<Vec<f64>>();
    assert_eq!(start_times, expected_start_times);
    let expected_end_times = vec![100., 3., 2., 1., 0.];
    let end_times = g.deme(0).end_times();
    let end_times = end_times
        .iter()
        .map(|size| f64::from(*size))
        .collect::<Vec<f64>>();
    assert_eq!(end_times, expected_end_times);
}

#[test]
fn defaults_deme_many_epochs_local_with_size_function_default() {
    let yaml = "
description: Modified from above to include default size_function
time_units: generations

demes:
- name: deme0
  defaults:
    epoch: {cloning_rate: 0.5, end_size: 2, selfing_rate: 0.1, start_size: 1, size_function: linear}
  epochs:
  - {end_time: 100, start_size: 1, end_size: 1}
  - {end_time: 3, size_function: exponential}
  - {end_time: 2}
  - {end_time: 1}
  - {end_time: 0}
";
    let g = demes::loads(yaml).unwrap();
    let size_functions = g
        .deme(0)
        .epochs()
        .iter()
        .map(|epoch| epoch.size_function())
        .collect::<Vec<SizeFunction>>();
    let expected_size_functions = vec![
        SizeFunction::Constant,
        SizeFunction::Exponential,
        SizeFunction::Linear,
        SizeFunction::Linear,
        SizeFunction::Linear,
    ];
    assert_eq!(size_functions, expected_size_functions);
}

// copied from demes-spec repo
#[test]
fn demelevel_defaults_epoch_03() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
  defaults:
    epoch: {end_time: 10}
- name: b
  epochs:
  - {start_size: 1, end_time: 90}
  - {start_size: 2, end_time: 50}
  - {start_size: 3}
  defaults:
    epoch: {end_time: 10}
";
    let g = demes::loads(yaml).unwrap();
    let deme = g.deme(0);
    assert_eq!(f64::from(deme.end_time()), 10.0);
    assert_eq!(f64::from(deme.start_time()), f64::INFINITY);
    let deme = g.deme(1);
    assert_eq!(f64::from(deme.end_time()), 10.0);
    let end_times = deme
        .end_times()
        .iter()
        .map(|time| f64::from(*time))
        .collect::<Vec<f64>>();
    assert_eq!(end_times, vec![90., 50., 10.]);
}

// copied from demes-spec repo
#[test]
fn infinity_03() {
    let yaml = "
time_units: generations
defaults:
  deme: {start_time: .inf}
demes:
  - name: A
    epochs:
      - start_size: 100
";
    let g = demes::loads(yaml).unwrap();
    let deme = g.deme(0);
    assert_eq!(f64::from(deme.start_time()), f64::INFINITY);
    assert_eq!(f64::from(deme.start_size()), 100.0);
    assert_eq!(f64::from(deme.end_size()), 100.0);
    assert_eq!(f64::from(deme.end_time()), 0.0);
}

// copied from demes-spec repo
#[test]
fn toplevel_defaults_deme_01() {
    let yaml = "
time_units: generations
defaults:
  deme:
    ancestors: [a, b, c]
    proportions: [0.1, 0.7, 0.2]
demes:
- name: a
  ancestors: []
  proportions: []
  epochs:
  - {start_size: 1}
- name: b
  ancestors: []
  proportions: []
  epochs:
  - {start_size: 1}
- name: c
  ancestors: []
  proportions: []
  epochs:
  - {start_size: 1}
- name: x
  start_time: 100
  epochs:
  - {start_size: 1}
- name: y
  start_time: 100
  epochs:
  - {start_size: 1}
- name: z
  start_time: 100
  epochs:
  - {start_size: 1}
";
    let g = demes::loads(yaml).unwrap();

    for deme in 0..3 {
        assert!(g.deme(deme).ancestors().is_empty());
        assert!(g.deme(deme).proportions().is_empty());
    }

    for deme in 3..6 {
        assert_eq!(g.deme(deme).ancestors().len(), 3);
        assert_eq!(g.deme(deme).proportions().len(), 3);
    }
}

// from demes-spec repo
#[test]
fn demlevel_defaults_epoch_01() {
    let yaml = "
time_units: generations
demes:
- name: a
  defaults:
    epoch: {start_size: 1}
- name: b
  epochs:
  - {end_time: 90}
  - {end_size: 100, end_time: 50}
  - {start_size: 100, end_size: 50}
  defaults:
    epoch: {start_size: 1}
";
    let g = demes::loads(yaml).unwrap();
    let start_sizes = g
        .deme(0)
        .start_sizes()
        .iter()
        .map(|size| f64::from(*size))
        .collect::<Vec<f64>>();
    assert_eq!(start_sizes, vec![1.0]);
    let start_sizes = g
        .deme(1)
        .start_sizes()
        .iter()
        .map(|size| f64::from(*size))
        .collect::<Vec<f64>>();
    assert_eq!(start_sizes, vec![1.0, 1.0, 100.0]);
}

// from demes-spec repo
#[test]
fn demlevel_defaults_epoch_06() {
    let yaml = "
time_units: generations
demes:
- name: a
  defaults:
    epoch: {end_size: 1}
- name: b
  epochs:
  - {end_time: 90}
  - {start_size: 100, end_time: 50}
  - {start_size: 1, end_size: 100}
  defaults:
    epoch: {end_size: 1}
";
    let g = demes::loads(yaml).unwrap();
    let start_sizes = g
        .deme(0)
        .start_sizes()
        .iter()
        .map(|size| f64::from(*size))
        .collect::<Vec<f64>>();
    assert_eq!(start_sizes, vec![1.0]);
    let start_sizes = g
        .deme(1)
        .start_sizes()
        .iter()
        .map(|size| f64::from(*size))
        .collect::<Vec<f64>>();
    assert_eq!(start_sizes, vec![1.0, 100.0, 1.0]);
}

#[test]
fn toplevel_defaults_epoch_03() {
    let yaml = "
time_units: generations
defaults:
  epoch: {end_time: 100}
demes:
- name: a
  epochs:
  - {start_size: 1}
- name: b
  epochs:
  - {start_size: 1}
- name: c
  epochs:
  - {start_size: 1}
- name: d
  ancestors: [a, b, c]
  proportions: [0.2, 0.3, 0.5]
  start_time: 100
  epochs:
  - {start_size: 1, end_time: 50}
  - {start_size: 2, end_time: 0}
- name: e
  ancestors: [a, b, c]
  proportions: [0.2, 0.3, 0.5]
  start_time: 100
  epochs:
  - {start_size: 1}
  - {start_size: 2, end_time: 10}
  defaults:
    epoch: {end_time: 50}
";
    let g = demes::loads(yaml).unwrap();

    for i in 0..3 {
        let end_times = g
            .deme(i)
            .end_times()
            .iter()
            .map(|time| f64::from(*time))
            .collect::<Vec<f64>>();
        assert_eq!(end_times, vec![100.]);
    }
    // deme d
    let end_times = g
        .deme(3)
        .end_times()
        .iter()
        .map(|time| f64::from(*time))
        .collect::<Vec<f64>>();
    assert_eq!(end_times, vec![50., 0.]);
    // deme e
    let end_times = g
        .deme(4)
        .end_times()
        .iter()
        .map(|time| f64::from(*time))
        .collect::<Vec<f64>>();
    assert_eq!(end_times, vec![50., 10.]);
}

#[test]
fn toplevel_metadata_01() {
    let yaml = r#"
time_units: generations
metadata:
  one: 1
  two: "two"
  three: [3, 3, 3]
  not_sure: null
  nested:
    nested:
      nested:
        now_im_done: "nested!"
demes:
  - name: a
    epochs:
    - start_size: 100
"#;
    let g = demes::loads(yaml).unwrap();
    assert!(g.metadata().is_some());
    let yaml_from_graph = serde_yaml::to_string(&g).unwrap();
    let g_from_yaml = demes::loads(&yaml_from_graph).unwrap();
    assert_eq!(g, g_from_yaml);
    let _json = serde_json::to_string(&g).unwrap();
    // NOTE: we cannot yet compare equality b/c
    // we do not have support for resolve, etc.?
    // The issue is that the internal deme_map
    // is used in PartialEq, which may be a mistake?
    // let _g_from_json: demes::Graph = serde_json::from_str(&json).unwrap();
    // assert_eq!(g, g_from_json);
}

#[test]
fn pulse_edge_case_02() {
    let yaml = "
time_units: generations
demes:
- name: deme1
  epochs:
  - {start_size: 1}
- name: deme2
  epochs:
  - {start_size: 1, end_time: 50}
- name: deme3
  ancestors: [deme2]
  epochs:
  - {start_size: 1}
pulses:
- {sources: [deme1], dest: deme3, proportions: [0.9], time: 50}
";
    let _ = demes::loads(yaml).unwrap();
}
