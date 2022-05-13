use demes::specification::{AsymmetricMigration, EndTime, MigrationRate, StartTime};

#[derive(Eq, PartialEq)]
struct ExpectedMigration {
    source: String,
    dest: String,
    rate: MigrationRate,
    start_time: StartTime,
    end_time: EndTime,
}

impl ExpectedMigration {
    fn new<
        N: ToString,
        M: TryInto<MigrationRate, Error = demes::DemesError>,
        S: TryInto<StartTime, Error = demes::DemesError>,
        E: TryInto<EndTime, Error = demes::DemesError>,
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
    let s = serde_yaml::to_string(&g).unwrap();
    let _ = demes::loads(&s).unwrap();
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

    let output = serde_yaml::to_string(&g).unwrap();
    // TODO: replace this with an explicit graph equality comparison
    let round_trip = demes::loads(&output).unwrap();
    assert!(round_trip
        .migrations()
        .iter()
        .all(|m| expected_resolved_migrations.contains(&ExpectedMigration::from(m.clone()))));
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
    let _ = demes::loads(yaml).unwrap();
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
    let _ = demes::loads(yaml).unwrap();
}
