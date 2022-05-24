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
    assert!(matches!(
        g.time_units(),
        demes::specification::TimeUnits::GENERATIONS
    ));
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
