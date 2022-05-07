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
