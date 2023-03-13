use anyhow::Result;

fn load_model() -> Result<demes::Graph> {
    let file = std::fs::File::open("examples/gutenkunst_ooa.yaml")?;
    let graph = demes::load(file)?;
    Ok(graph)
}

// Convert the demes graph into a ForwardGraph and iterate the model
// starting from forward time 0.
//
// # Notes
//
// The API shown here gives flexibility at the cost of ergonomics:
//
// * We have to call update_state both before and during iteration.
//   This allows us to start a model at any valid time.
//   It also allows us to re-use a ForwardGraph, iterating from different
//   starting points, etc..
// * An alternative (and unimplemented) idea is to consume the ForwardGraph
//   into an iterator that handles the state updates.
//   By implementing IntoIterator for ForwardGraph, we could say:
//   for time in forward_graph { ... } and not have to worry about
//   calling update_state at the end of the loop body.
// * The API currently returns Option<_> for deme sizes, etc..
//   It would be nice to find a way to avoid that.
fn iterate_model(graph: demes::Graph, burnin: i32) -> Result<demes_forward::ForwardTime> {
    // Convert the backwards time model
    // to a forward-time representation with
    // some generations of burn-in.
    //
    // The forward graph is mutable because
    // we will update its internal state
    // during iteration.
    //
    // The final argument, None, means to apply
    // no rounding when converting times into generations.
    // None is safe for this model, but may not be so generally.
    // See the demes-rs docs at https://docs.rs/demes for details on rounding methods.
    //
    // NOTE: the implementation of rounding in demes is currently an enum.
    // In the future, it may become a trait, which would break API here
    // but allow for more flexibility in client code.
    let mut forward_graph = demes_forward::ForwardGraph::new_discrete_time(graph, burnin)?;

    // Update the internal model state
    // to parental generation 0, meaning
    // that the first offspring will have
    // birth times of generation 1.
    forward_graph.update_state(0)?;

    for time in forward_graph.time_iterator() {
        // time refers to a parental generation.
        // Therefore, when we have iterated to the time point
        // where the final generation are now parents, there
        // are no offspring and forward_graph.offspring_deme_sizes()
        // returns None.
        if let Some(offspring_deme_sizes) = forward_graph.offspring_deme_sizes() {
            // Get the parent deme sizes.
            // Given our previous "if let ...", this statement cannot/should not
            // return None, and we will panic! if it does.
            let parental_deme_sizes = forward_graph
                .parental_deme_sizes()
                .unwrap_or_else(|| panic!("expected parental deme sizes at time {time}"));

            // The deme size slices have lengths equal to the total
            // number of demes in the graph.
            assert_eq!(
                offspring_deme_sizes.len(),
                forward_graph.num_demes_in_model()
            );
            assert_eq!(
                parental_deme_sizes.len(),
                forward_graph.num_demes_in_model()
            );

            // Get the selfing and cloning rates for each deme.
            // The order is the same as the order of demes in the graph.
            // This model has no selfing/cloning, but we include the accesses
            // for completeness.
            let selfing_rates = forward_graph
                .selfing_rates()
                .unwrap_or_else(|| panic!("expected selfing rates"));
            let cloning_rates = forward_graph
                .cloning_rates()
                .unwrap_or_else(|| panic!("expected cloning rates"));

            assert_eq!(selfing_rates.len(), forward_graph.num_demes_in_model());
            assert_eq!(cloning_rates.len(), forward_graph.num_demes_in_model());

            // Iterate over offspring deme indexes
            for (offspring_deme_index, offspring_deme_size) in
                offspring_deme_sizes.iter().enumerate()
            {
                let ancestry_proportions = forward_graph
                    .ancestry_proportions(offspring_deme_index)
                    .unwrap_or_else(|| {
                        panic!(
                            "expected ancestry proportions for offspring deme {offspring_deme_index} at time {time}",
                        )
                    });
                if offspring_deme_size > &0.0 {
                    // If an the offspring deme is extant (size > 0),
                    // then any ancestral deme with a proportion > 0 must
                    // have size > 0.
                    // Further, the sum of all ancestry proportions into
                    // the offspring deme must sum to ~1.0.
                    assert!((ancestry_proportions.iter().sum::<f64>() - 1.0).abs() <= f64::EPSILON);
                    for (ancestor, proportion) in ancestry_proportions.iter().enumerate() {
                        if proportion > &0.0 {
                            assert!(parental_deme_sizes[ancestor] > 0.0);
                        }
                        // NOTE: is incorrect to assert that the ancestor
                        // size is 0 if the ancestry proportion of that ancestor
                        // into offspring_deme is 0!
                        // The ancestor deme could exist but only as an ancestor
                        // of another offspring deme in the graph.
                    }
                } else {
                    // An offspring deme with size 0 has no ancestry.
                    assert_eq!(ancestry_proportions.iter().sum::<f64>(), 0.0);
                }
            }
        }

        // Update the internal state to the next time point.
        forward_graph.update_state(time + 1.into())?;
    }

    Ok(forward_graph
        .last_time_updated()
        .expect("expected Some(ForwardTime)"))
}

fn do_work(burnin: i32) -> Result<()> {
    // Read in the YAML model, giving
    // the backwards-time "MDM" representation
    // of the resolved graph.
    let graph = load_model()?;

    println!("The input model time units are: {}", graph.time_units());
    let generation_time = graph.generation_time();
    println!(
        "The generation time is: {} {} per generation",
        generation_time,
        graph.time_units()
    );
    let mut most_ancient_finite_epoch_start_time = demes::Time::from(0.0);
    for deme in graph.demes() {
        for epoch in deme.epochs() {
            let start_time = epoch.start_time();
            if f64::from(start_time).is_finite() {
                most_ancient_finite_epoch_start_time =
                    std::cmp::max(most_ancient_finite_epoch_start_time, epoch.start_time());
            }
        }
    }
    println!(
        "The most ancient start time of any deme's epoch is: {} {} ago",
        most_ancient_finite_epoch_start_time,
        graph.time_units(),
    );
    println!(
        "The most recent end time of any deme is: {} {} ago",
        graph.most_recent_deme_end_time(),
        graph.time_units()
    );

    // in generations
    let most_ancient_finite_epoch_start_time_generations =
        f64::from(most_ancient_finite_epoch_start_time) / f64::from(generation_time);
    println!(
        "The most ancient start time of any deme's epoch is: {most_ancient_finite_epoch_start_time_generations} generations ago",
    );

    println!("The burn-in time of the model is {burnin} generations");

    let time_spent_in_graph_after_burnin = most_ancient_finite_epoch_start_time_generations
        - f64::from(graph.most_recent_deme_end_time()) / f64::from(generation_time);

    println!(
        "The graph will iterate over {burnin} burn-in generations + {time_spent_in_graph_after_burnin} generations",
    );

    println!(
        "Therefore, the birth time of the final generation is {}",
        f64::from(burnin) + time_spent_in_graph_after_burnin + 1.0
    );

    let last_offspring_birth_time = iterate_model(graph, burnin)?;

    println!("The last offspring birth time after iteration is {last_offspring_birth_time}",);

    Ok(())
}

fn main() {
    do_work(10000).unwrap();
}

#[test]
fn test_iterate_gutenkunst_ooa() {
    do_work(10000).unwrap();
}
