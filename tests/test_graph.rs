use demes::specification::{AsymmetricMigration, MigrationRate, Time};

#[derive(Eq, PartialEq)]
struct ExpectedMigration {
    source: String,
    dest: String,
    rate: MigrationRate,
    start_time: Time,
    end_time: Time,
}

impl ExpectedMigration {
    fn new<
        N: ToString,
        M: TryInto<MigrationRate, Error = demes::DemesError>,
        S: TryInto<Time, Error = demes::DemesError>,
        E: TryInto<Time, Error = demes::DemesError>,
    >(
        source: N,
        dest: N,
        rate: M,
        start_time: S,
        end_time: E,
    ) -> Result<Self, demes::DemesError> {
        let rate = rate.try_into()?;
        let start_time = start_time.try_into()?;
        let end_time = end_time.try_into()?;
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
    graph: &demes::specification::Graph,
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
    assert!(matches!(
        g.time_units(),
        demes::specification::TimeUnits::YEARS,
    ));
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
        demes::specification::Time::try_from(501.).unwrap(),
        demes::specification::Time::try_from(501.).unwrap(),
        demes::specification::Time::try_from(500.).unwrap(),
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
    assert!(matches!(sf, demes::specification::SizeFunction::CONSTANT));
    let sf = graph.deme(0).epochs()[1].size_function();
    assert!(matches!(sf, demes::specification::SizeFunction::LINEAR));
}
