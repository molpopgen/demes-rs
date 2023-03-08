fn yaml_non_integer() -> String {
    let yaml = "
time_units: generations
demes:
- name: deme1
  start_time: .inf
  epochs:
  - {end_size: 99.99000049998334, end_time: 8000.0, start_size: 99.99000049998334}
  - {end_size: 100.0, end_time: 4000.0, start_size: 99.99000049998334}
  - {end_size: 100, end_time: 0, start_size: 100.0}
migrations: []
";
    yaml.to_owned()
}

fn yaml_all_integer() -> String {
    // Same as above, but we've manually rounded everything.
    let yaml = "
time_units: generations
demes:
- name: deme1
  start_time: .inf
  epochs:
  - {end_size: 100.0, end_time: 8000.0, start_size: 100.}
  - {end_size: 100.0, end_time: 4000.0, start_size: 100.}
  - {end_size: 100, end_time: 0, start_size: 100.0}
migrations: []
";
    yaml.to_owned()
}

#[test]
fn test_has_non_integer_sizes() {
    let graph = demes::loads(&yaml_non_integer()).unwrap();
    assert!(graph.has_non_integer_sizes());
    let graph = demes::loads(&yaml_all_integer()).unwrap();
    assert!(!graph.has_non_integer_sizes());
}
