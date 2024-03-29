#[test]
#[should_panic]
fn exponential_size_function_with_no_size_change() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
        end_size: 1000
        size_function: exponential
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn linear_size_function_with_no_size_change() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
        end_size: 1000
        size_function: linear
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn test_invalid_epoch_field() {
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
      - start_time: 1000
";
    let g = demes::loads(yaml).unwrap();
    assert_eq!(g.num_demes(), 2);
}

#[test]
#[should_panic]
fn missing_generation_time() {
    let yaml = "
time_units: years
demes:
  - name: A
    epochs:
      - start_size: 1000
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn ancestors_and_proportions_different_lengths() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
        end_time: 1000
  - name: B
    proportions: [1.0]
    epochs:
      - start_size: 2000
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn invalid_proportion_sum() {
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
    proportions: [0.25, 0.5]
    start_time: 1000
    epochs:
      - start_size: 50
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn missing_time_units() {
    let yaml = "
demes:
  - name: A
    epochs:
      - start_size: 1000
";
    let g = demes::loads(yaml).unwrap();
    assert!(matches!(g.time_units(), demes::TimeUnits::Generations));
}

#[test]
#[should_panic]
fn too_few_demes_symmetric_migration() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    epochs:
      - start_size: 1000
migrations:
  - demes: [A]
    rate: 0.125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn deme_listed_more_than_once_symmetric_migration() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    epochs:
      - start_size: 1000
migrations:
  - demes: [A, A]
    rate: 0.125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn deme_does_not_exist_symmetric_migration() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    epochs:
      - start_size: 1000
migrations:
  - demes: [A, C]
    rate: 0.125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn dest_deme_does_not_exist_asymmetric_migration() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    epochs:
      - start_size: 1000
migrations:
  - source: A
    dest: C
    rate: 0.125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn source_deme_does_not_exist_asymmetric_migration() {
    let yaml = "
time_units: generations
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    epochs:
      - start_size: 1000
migrations:
  - source: C
    dest: A
    rate: 0.125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn demes_do_not_overlap_in_time_asymmetric_migration() {
    let yaml = "
time_units: generations
description: demes B and C do not co-exist, so how can they migrate?
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    ancestors: [A]
    start_time: 200
    epochs:
      - start_size: 1000
        end_time: 150
  - name: C
    ancestors: [A]
    start_time: 100
    epochs:
      - start_size: 1000
        end_time: 50
migrations:
  - source: C
    dest: B
    rate: 0.125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn demes_do_not_overlap_in_time_pulses() {
    let yaml = "
time_units: generations
description: The pulse is at time 125 from B to C. Neither deme exists then.
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    ancestors: [A]
    start_time: 200
    epochs:
      - start_size: 1000
        end_time: 150
  - name: C
    ancestors: [A]
    start_time: 100
    epochs:
      - start_size: 1000
        end_time: 50
pulses:
  - sources: [B]
    dest: C
    proportions: [0.125]
    time: 125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn pulses_proportions_zero() {
    let yaml = "
time_units: generations
description: The pulse is at time 125 from B to C. Neither deme exists then.
demes:
  - name: A
    epochs:
      - start_size: 1000
  - name: B
    ancestors: [A]
    start_time: 200
    epochs:
      - start_size: 1000
  - name: C
    ancestors: [A]
    start_time: 200
    epochs:
      - start_size: 1000
pulses:
  - sources: [B]
    dest: C
    proportions: [0.0]
    time: 125
";
    let _ = demes::loads(yaml).unwrap();
}

#[test]
#[should_panic]
fn size_function_bad_default() {
    let yaml = "
time_units: generations
description: setting default size function to constant is bad.
defaults:
  epoch:
    start_size: 5000
    size_function: constant
demes:
  - name: X
    epochs:
      - end_time: 1000
      - end_size: 100

";
    let _ = demes::loads(yaml).unwrap();
}

// copied from demes-spec repo
#[test]
fn infinity_03_bad_default() {
    let yaml = "
time_units: generations
description: modified from demes-spec example to have invalid default start time
defaults:
  deme: {start_time: 100.0}
demes:
  - name: A
    epochs:
      - start_size: 100
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

// copied from demes-spec repo
#[test]
fn bad_ancestors_06() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 100, end_time: 50}
- name: b
  epochs:
  - {start_size: 100, end_time: 50}
- name: c
  ancestors: [c, b]
  proportions: [0.5, 0.5]
  start_time: 100
  epochs:
  - {start_size: 100}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

// copied from demes-spec repo
#[test]
fn bad_ancestors_07() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 100, end_time: 50}
- name: b
  epochs:
  - {start_size: 100, end_time: 50}
- name: c
  ancestors: [d, b]
  proportions: [0.5, 0.5]
  start_time: 100
  epochs:
  - {start_size: 100}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

// copied from demes-spec repo
#[test]
fn bad_migrations_11() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
- name: b
  epochs:
  - {start_size: 1}
migrations:
- {source: a, dest: b}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migrations_12() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
- name: b
  epochs:
  - {start_size: 1}
migrations:
- demes: [a, b]
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migrations_20() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 100}
- name: b
  epochs:
  - {start_size: 100}
migrations:
- rate: 0.1
  source: a
  dest: a
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migrations_21() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs: 
  - start_size: 1
- name: b
  epochs: 
  - start_size: 1
migrations:
- {source: a, dest: b, rate: 0.1, foo: 0.2}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::YamlError(_))),
    }
}

#[test]
fn bad_pulses_14() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
- name: b
  epochs:
  - {start_size: 1}
pulses:
- {sources: [a], proportions: [0.1], time: 100}
    ";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::PulseError(_))),
    }
}

#[test]
fn bad_pulses_19() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - start_size: 1
- name: b
  epochs:
  - start_size: 1
pulses:
- {sources: [a], dest: b, time: 100, proportions: [0.1], foo: 0.2}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::YamlError(_))),
    }
}

#[test]
fn bad_migration_start_end_times_03() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 100}
- name: b
  epochs:
  - {start_size: 100}
migrations:
- rate: 0.1
  source: a
  dest: b
  start_time: 0
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migration_start_end_times_07() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 100, end_time: 100}
- name: b
  ancestors: [a]
  start_time: 200
  epochs:
  - {start_size: 100}
migrations:
- rate: 0.1
  demes: [a, b]
  start_time: 250
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migration_start_end_times_10() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 100}
- name: b
  epochs:
  - {start_size: 100}
migrations:
- rate: 0.1
  source: a
  dest: b
  end_time: .inf
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migration_start_end_times_15() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 100, end_time: 100}
- name: b
  ancestors: [a]
  start_time: 200
  epochs:
  - {start_size: 100}
migrations:
- rate: 0.1
  demes: [a, b]
  start_time: 150
  end_time: 150
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn overlapping_migrations_01() {
    let yaml = "
time_units: generations
demes:
- name: A
  epochs:
  - {start_size: 1}
- name: B
  epochs:
  - {start_size: 1}
migrations:
- {rate: 0.01, source: A, dest: B}
- {rate: 0.02, source: A, dest: B, start_time: 10}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_pulse_source_dest_01() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
- name: b
  epochs:
  - {start_size: 1}
pulses:
- {sources: [a], dest: a, proportions: [0.1], time: 100}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::PulseError(_))),
    }
}

#[test]
fn bad_pulse_source_dest_07() {
    let yaml = "
time_units: generations
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
pulses:
- {sources: [a, a], dest: c, proportions: [0.1, 0.1], time: 100}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::PulseError(_))),
    }
}

#[test]
fn bad_pulse_time_02() {
    let yaml = "
time_units: generations
defaults:
  epoch: {start_size: 100}
demes:
- {name: A}
- name: B
  ancestors: [A]
  start_time: 100
  epochs:
  - {end_time: 50}
pulses:
- {sources: [A], dest: B, proportions: [0.1], time: 50}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::PulseError(_))),
    }
}

#[test]
fn bad_pulse_time_03() {
    let yaml = "
time_units: generations
defaults:
  epoch: {start_size: 100}
demes:
- {name: A}
- name: B
  ancestors: [A]
  start_time: 100
  epochs:
  - {end_time: 50}
pulses:
- {sources: [B], dest: A, proportions: [0.1], time: 100}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::PulseError(_))),
    }
}

#[test]
fn missing_demes_02() {
    let yaml = "
time_units: generations
demes: []
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

#[test]
fn bad_toplevel_defaults_02() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
defaults: []
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::YamlError(_))),
    }
}

#[test]
fn bad_toplevel_metadata_01() {
    let yaml = "
time_units: generations
metadata: 
demes:
  - name: a
    epochs:
    - start_size: 100
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::YamlError(_))),
    }
}

#[test]
fn bad_toplevel_defaults_06() {
    let yaml = "
time_units: generations
defaults: {rate: 0.1, proportion: 0.1}
demes:
- name: a
  epochs:
  - {start_size: 1}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::YamlError(_))),
    }
}

#[test]
fn bad_demelevel_defaults_02() {
    let yaml = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
  defaults: []
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::YamlError(_))),
    }
}

#[test]
fn bad_generation_time_08() {
    let yaml = "
time_units: generations
generation_time: 0
demes:
  - name: a
    epochs:
    - start_size: 100
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::GraphError(_))),
    }
}

#[test]
fn bad_generation_time_09() {
    let yaml = "
time_units: generations
generation_time: 13
demes:
- name: A
  epochs:
  - {start_size: 2000, end_time: 100}
  - {start_size: 100}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::GraphError(_))),
    }
}

#[test]
fn bad_deme_name_01() {
    let yaml = r#"
time_units: generations
demes:
  - name: ""
    epochs:
    - start_size: 100
"#;
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

#[test]
fn bad_deme_name_02() {
    let yaml = r#"
time_units: generations
demes:
  - name: a b
    epochs:
    - start_size: 100
"#;
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

#[test]
fn bad_deme_name_03() {
    let yaml = r#"
time_units: generations
demes:
  - name: a-b
    epochs:
    - start_size: 100
"#;
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

#[test]
fn bad_deme_name_04() {
    let yaml = r#"
time_units: generations
demes:
  - name: 900
    epochs:
    - start_size: 100
"#;
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

#[test]
fn bad_deme_name_05() {
    let yaml = r#"
time_units: generations
demes:
  - name: "\u0669"
    epochs:
    - start_size: 100
"#;
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

#[test]
fn bad_deme_name_07() {
    let yaml = r#"
time_units: generations
demes:
  - name: "π ٩"
    epochs:
    - start_size: 100
"#;
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::DemeError(_))),
    }
}

#[test]
fn bad_migration_rates_sum_01() {
    let yaml = "
time_units: generations
defaults:
  epoch: {start_size: 1}
demes:
- {name: A}
- {name: B}
- {name: C}
- {name: D}
migrations:
- {rate: 0.5, source: B, dest: A}
- {rate: 0.5, source: C, dest: A}
- {rate: 1e-03, source: D, dest: A}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migration_rates_sum_02() {
    let yaml = "
time_units: generations
defaults:
  epoch: {start_size: 1}
demes:
- {name: A}
- {name: B}
- {name: C}
- {name: D}
migrations:
- {rate: 0.6, source: C, dest: A, start_time: 100, end_time: 50}
- {rate: 0.6, source: B, dest: A, start_time: 200, end_time: 100}
- {rate: 0.6, source: D, dest: A, start_time: 60, end_time: 20}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}

#[test]
fn bad_migration_rates_sum_03() {
    let yaml = "
time_units: generations
defaults:
  epoch: {start_size: 1}
demes:
- {name: A}
- {name: B}
- {name: C}
migrations:
- rate: 0.6
  demes: [A, B]
  start_time: 100
- {rate: 0.6, source: C, dest: A}
";
    match demes::loads(yaml) {
        Ok(_) => panic!("expected Err!"),
        Err(e) => assert!(matches!(e, demes::DemesError::MigrationError(_))),
    }
}
