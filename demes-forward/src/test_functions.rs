pub fn update_ancestry_proportions(
    sources: &[usize],
    source_proportions: &[f64],
    ancestry_proportions: &mut [f64],
) {
    assert_eq!(sources.len(), source_proportions.len());
    let sum = source_proportions.iter().fold(0.0, |a, b| a + b);
    ancestry_proportions.iter_mut().for_each(|a| *a *= 1. - sum);
    sources
        .iter()
        .zip(source_proportions.iter())
        .for_each(|(source, proportion)| ancestry_proportions[*source] += proportion);
}

pub fn ancestry_proportions_from_graph(
    graph: &crate::ForwardGraph,
    child_deme: usize,
) -> Option<Vec<f64>> {
    graph.offspring_deme_sizes()?;

    let mut rv = vec![0.0; graph.offspring_deme_sizes().unwrap().len()];

    let deme = graph.demes_graph().get_deme(child_deme).unwrap();
    let bwtime = graph
        .time_to_backward(graph.last_time_updated().unwrap())
        .unwrap()
        .unwrap();
    if bwtime > deme.start_time() || bwtime < deme.end_time() {
        return Some(rv);
    }

    if !deme.ancestor_indexes().is_empty() && bwtime == deme.start_time() {
        for (a, p) in deme
            .ancestor_indexes()
            .iter()
            .zip(deme.proportions().iter())
        {
            rv[*a] = f64::from(*p);
        }
    } else {
        rv[child_deme] = 1.0;
    }

    let mut sources: Vec<usize> = vec![];
    let mut source_proportions: Vec<f64> = vec![];

    let bwtime: f64 = graph
        .time_to_backward(graph.last_time_updated().unwrap())
        .unwrap()
        .unwrap()
        .into();
    // NOTE: we subract 1 because:
    // * "last time updated" refers to the birth time of the current parents.
    // * this fn is trying to get the ancestry proportions of the next generation of children.
    // * the children are 1 generation closer to the present, so we subtract 1!
    let bwtime = demes::Time::try_from(bwtime - 1.0).unwrap();
    for p in graph
        .demes_graph()
        .pulses()
        .iter()
        .filter(|p| p.time() == f64::from(bwtime))
    {
        sources.clear();
        source_proportions.clear();
        let dest = graph
            .demes_graph()
            .demes()
            .iter()
            .position(|d| d.name() == p.dest())
            .unwrap();
        if dest == child_deme {
            for (s, d) in p.sources().iter().zip(p.proportions().iter()) {
                let source = graph
                    .demes_graph()
                    .demes()
                    .iter()
                    .position(|d| d.name() == s)
                    .unwrap();
                sources.push(source);
                source_proportions.push(f64::from(*d));
            }
            update_ancestry_proportions(&sources, &source_proportions, &mut rv);
        }
    }

    sources.clear();
    source_proportions.clear();

    for m in graph
        .demes_graph()
        .migrations()
        .iter()
        .filter(|m| bwtime >= m.end_time() && bwtime < m.start_time())
    {
        let d = graph
            .demes_graph()
            .demes()
            .iter()
            .position(|deme| deme.name() == m.dest())
            .unwrap();
        if d == child_deme {
            let s = graph
                .demes_graph()
                .demes()
                .iter()
                .position(|deme| deme.name() == m.source())
                .unwrap();
            sources.push(s);
            source_proportions.push(f64::from(m.rate()));
        }
    }
    update_ancestry_proportions(&sources, &source_proportions, &mut rv);

    Some(rv)
}

pub fn test_model_duration(graph: &mut crate::ForwardGraph) {
    for time in graph.time_iterator() {
        graph.update_state(time).unwrap();
        // assert!(graph.parental_demes().is_some(), "{}", time);
        assert!(
            graph.num_extant_parental_demes() > 0,
            "{:?} {:?}",
            time,
            graph.end_time()
        );
        assert!(graph
            .parental_deme_sizes()
            .unwrap()
            .iter()
            .any(|size| size > &0.0));
        if time == graph.end_time() - 1.0.into() {
            assert!(graph.offspring_deme_sizes().is_none(), "time = {time:?}");
        } else {
            assert!(graph.offspring_deme_sizes().is_some(), "time = {time:?}");
        }
    }
}
