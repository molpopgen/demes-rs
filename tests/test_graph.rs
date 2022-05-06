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
