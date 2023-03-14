use demes_forward::demes;

#[derive(Debug)]
struct ModelFirstLast {
    first: Option<demes_forward::ForwardTime>,
    last: Option<demes_forward::ForwardTime>,
}

pub fn four_deme_model() -> demes::Graph {
    let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
    - start_size: 100
      end_time: 50
 - name: B
   ancestors: [A]
   epochs:
    - start_size: 100
 - name: C
   ancestors: [A]
   epochs:
    - start_size: 100
      end_time: 49
 - name: D
   ancestors: [C, B]
   proportions: [0.5, 0.5]
   start_time: 49
   epochs:
    - start_size: 50
";
    demes::loads(yaml).unwrap()
}

fn iterate_all_generations(graph: &mut demes_forward::ForwardGraph) -> ModelFirstLast {
    let mut first_time_visited = None;
    let mut last_time_visited = None;
    for time in graph.time_iterator() {
        match first_time_visited {
            None => first_time_visited = Some(time),
            Some(_) => (),
        }
        if time == demes_forward::ForwardTime::from(0.0) {
            assert!(graph.last_time_updated().is_none());
        }
        last_time_visited = Some(time);
        graph.update_state(time).unwrap();
        assert_eq!(graph.last_time_updated(), Some(time));
        match graph.offspring_deme_sizes() {
            Some(child_deme_sizes) => {
                assert!(time < graph.end_time() - 1.0.into());
                assert!(graph.any_extant_parental_demes());
                assert!(graph.any_extant_offspring_demes());
                let parental_deme_sizes = graph.parental_deme_sizes().unwrap();
                let selfing_rates = graph.selfing_rates().unwrap();
                let cloning_rates = graph.cloning_rates().unwrap();
                assert_eq!(child_deme_sizes.len(), graph.num_demes_in_model());
                assert_eq!(parental_deme_sizes.len(), graph.num_demes_in_model());
                assert_eq!(selfing_rates.len(), graph.num_demes_in_model());
                assert_eq!(cloning_rates.len(), graph.num_demes_in_model());

                // Stress-test that a deme > no. demes in model returns None
                assert!(graph
                    .ancestry_proportions(graph.num_demes_in_model())
                    .is_none());
                for i in 0..graph.num_demes_in_model() {
                    if selfing_rates[i] > 0.0 {
                        assert!(child_deme_sizes[i] > 0.0);
                    }
                    if cloning_rates[i] > 0.0 {
                        assert!(child_deme_sizes[i] > 0.0);
                    }
                    let ancestry_proportions = graph.ancestry_proportions(i).unwrap();
                    for j in 0..ancestry_proportions.len() {
                        if ancestry_proportions[j] > 0.0 {
                            assert!(parental_deme_sizes[j] > 0.0);
                            assert!(
                                child_deme_sizes[i] > 0.0,
                                "{time:?}, {i:?} => {child_deme_sizes:?}"
                            );
                        }
                    }
                }
            }
            None => {
                assert!(!graph.any_extant_offspring_demes());
                assert!(graph.selfing_rates().is_none());
                assert!(graph.cloning_rates().is_none());
                assert!(time <= graph.end_time() - 1.0.into());
            }
        }
    }
    ModelFirstLast {
        first: first_time_visited,
        last: last_time_visited,
    }
}

#[test]
fn test_ancestry_proportions_after_deme_has_gone_extinct_and_before_extant() {
    let demes_graph = four_deme_model();
    let mut graph = demes_forward::ForwardGraph::new_discrete_time(demes_graph, 100).unwrap();
    graph.update_state(100).unwrap();

    // Deme A is extinct and D hasn't "come alive" yet
    for deme in [0_usize, 3_usize] {
        assert_eq!(graph.offspring_deme_sizes().unwrap()[deme], 0.);
        let ancestry_proportions = graph.ancestry_proportions(deme).unwrap();
        assert_eq!(ancestry_proportions.len(), graph.num_demes_in_model());
        let sum_ancestry_proportions: f64 = ancestry_proportions.iter().sum();
        assert_eq!(sum_ancestry_proportions, 0.0);
    }
}

#[test]
fn test_four_deme_model_pub_api_only() {
    let demes_graph = four_deme_model();
    let mut graph = demes_forward::ForwardGraph::new_discrete_time(demes_graph, 100).unwrap();
    let last_time = iterate_all_generations(&mut graph);
    assert_eq!(
        last_time.last,
        Some(demes_forward::ForwardTime::from(150.0))
    );
    assert_eq!(last_time.first, Some(demes_forward::ForwardTime::from(0.0)));
}

#[test]
fn test_four_deme_model_pub_api_only_start_after_zero() {
    let demes_graph = four_deme_model();
    let mut graph = demes_forward::ForwardGraph::new_discrete_time(demes_graph, 100).unwrap();
    graph.update_state(50.0).unwrap();
    let last_time = iterate_all_generations(&mut graph);
    assert_eq!(
        last_time.last,
        Some(demes_forward::ForwardTime::from(150.0))
    );
    assert_eq!(
        last_time.first,
        Some(demes_forward::ForwardTime::from(50.0))
    );
}

#[test]
fn gutenkunst2009() {
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
    let demes_graph = demes::loads(yaml).unwrap();
    let mut graph = demes_forward::ForwardGraph::new_discrete_time(demes_graph, 0).unwrap();
    // graph.update_state(0.0).unwrap();
    let _last_time = iterate_all_generations(&mut graph);
}

#[test]
fn jacobs2019() {
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
    let demes_graph = demes::loads(yaml).unwrap();
    let mut graph = demes_forward::ForwardGraph::new_discrete_time(demes_graph, 0).unwrap();
    // graph.update_state(0.0).unwrap();
    let _last_time = iterate_all_generations(&mut graph);
}

#[test]
fn test_zero_length_model() {
    let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
";
    let demes_graph = demes::loads(yaml).unwrap();
    let mut graph = demes_forward::ForwardGraph::new_discrete_time(demes_graph, 0).unwrap();
    assert_eq!(graph.end_time(), 1.0.into());
    let last_time = iterate_all_generations(&mut graph);
    let first = last_time.first.expect("expected Some(time) for first");
    assert_eq!(first, 0.0.into());
    let last = last_time.last.expect("expected Some(time) for last");
    assert_eq!(last, 0.0.into());
}

#[test]
fn test_model_length() {
    let yaml = "
time_units: generations
demes:
 - name: A
   epochs:
   - start_size: 100
     end_time: 50
   - start_size: 200
";
    let demes_graph = demes::loads(yaml).unwrap();
    let mut graph = demes_forward::ForwardGraph::new_discrete_time(demes_graph, 100).unwrap();
    assert_eq!(graph.end_time(), 151.0.into());
    let last_time = iterate_all_generations(&mut graph);
    let first = last_time.first.expect("expected Some(time) for first");
    assert_eq!(first, 0.0.into());
    let last = last_time.last.expect("expected Some(time) for last");
    assert_eq!(last, 150.0.into());
}

#[test]
fn test_reject_non_integer_start_size() {
    let yaml = "
time_units: generations
demes:
- name: deme1
  start_time: .inf
  epochs:
  - {end_size: 100.0, end_time: 8000.0, start_size: 100.0}
  - {end_size: 100.0, end_time: 4000.0, start_size: 99.99000049998334}
  - {end_size: 100, end_time: 0, start_size: 100.0}
migrations: []
";
    let demes_graph = demes::loads(yaml).unwrap();
    assert!(demes_forward::ForwardGraph::new_discrete_time(demes_graph, 100,).is_err());
}

#[test]
fn test_reject_non_integer_end_size() {
    let yaml = "
time_units: generations
demes:
- name: deme1
  start_time: .inf
  epochs:
  - {end_size: 100.0, end_time: 8000.0, start_size: 100.0}
  - {end_size: 99.99000049998334, end_time: 4000.0, start_size: 100.0}
  - {end_size: 100, end_time: 0, start_size: 100.0}
migrations: []
";
    let demes_graph = demes::loads(yaml).unwrap();
    assert!(demes_forward::ForwardGraph::new_discrete_time(demes_graph, 100,).is_err());
}

#[test]
fn test_initial_sizes_when_model_ends_prior_to_time_zero() {
    let yaml_with_default_end_sizes = "
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
    let yaml_shift_model_ten_gens_towards_present = "
time_units: generations
demes:
- name: a
  epochs:
  - {start_size: 1}
  defaults:
    epoch: {end_time: 0}
- name: b
  epochs:
  - {start_size: 1, end_time: 80}
  - {start_size: 2, end_time: 40}
  - {start_size: 3}
";
    let yaml_explicit_end_times = "
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
  - {start_size: 3, end_time: 10}
";
    for yaml_model in [
        yaml_shift_model_ten_gens_towards_present,
        yaml_with_default_end_sizes,
        yaml_explicit_end_times,
    ] {
        for burnin in (0..50).rev() {
            let demes_graph = demes::loads(yaml_model).unwrap();
            let mut graph =
                demes_forward::ForwardGraph::new_discrete_time(demes_graph, burnin).unwrap();
            graph.update_state(0.0).unwrap();
            let parental_sizes = graph.parental_deme_sizes().unwrap();
            assert!(
                parental_sizes.iter().all(|x| x == &1.0),
                "failed with burnin = {burnin} and model variant {yaml_model}"
            );
        }
    }
}
