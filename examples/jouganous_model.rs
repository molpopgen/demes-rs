use anyhow::Result;

fn load_yaml(path: &str) -> Result<demes::Graph> {
    let file = std::fs::File::open(path)?;
    let graph = demes::load(file)?;
    Ok(graph)
}

fn display_graph<T>(graph: &T)
where
    T: std::fmt::Display,
{
    println!("We can print the Graph as a YAML string:\n");
    println!("{graph}");
}

fn graph_api_examples(graph: &demes::Graph) {
    println!("Example of the graph API:\n");

    println!("Description: {}", graph.description().unwrap_or("None"));

    // DOI data are stored in a vector.
    // The API provides an iterator
    let num_doi = graph.doi().count();

    match num_doi {
        0 => println!("There is no DOI information for this graph"),
        _ => {
            println!("DOI:");
            graph.doi().for_each(|doi| println!("\t{doi}"));
        }
    }

    println!("The time units are: {}", graph.time_units());

    println!("The generation time is: {}", graph.generation_time());
}

fn iterate_demes_and_epochs(graph: &demes::Graph) {
    println!("\nIterate over demes and their epochs:\n");
    // Get a &[demes::Deme] (slice of demes)
    for deme in graph.demes() {
        println!("Deme {}: {}", deme.name(), deme.description());
        println!("\tstart_time: {}", deme.start_time());
        println!("\tend_time: {}", deme.end_time());
        println!("\ttime_interval: {}", deme.time_interval());
        println!("\tstart_size: {}", deme.start_size());
        println!("\tend_size: {}", deme.end_size());

        // deme.epochs returns &[demes::Epoch] (slice of epochs),
        // which we then enumerate over.
        for (i, epoch) in deme.epochs().iter().enumerate() {
            println!("\tepoch {i}:");
            println!("\t\tstart_time: {}", epoch.start_time());
            println!("\t\tend_time: {}", epoch.end_time());
            println!("\t\ttime_interval: {}", epoch.time_interval());
            println!("\t\tstart_size: {}", epoch.start_size());
            println!("\t\tend_size: {}", epoch.end_size());
            println!("\t\tsize_function: {}", epoch.size_function());
        }
    }
}

fn iterate_migrations(graph: &demes::Graph) {
    println!("\nIterate over asymmetric migrations:\n");
    // Enumerate the &[demes::AsymmetricMigration]
    for (i, migration) in graph.migrations().iter().enumerate() {
        println!("migration {i}");
        println!("\tstart_time: {}", migration.start_time());
        println!("\tend_time: {}", migration.end_time());
        println!("\ttime_interval: {}", migration.time_interval());
        println!("\tsource: {}", migration.source());
        println!("\tdest: {}", migration.dest());
    }
}

fn iterate_pulses(graph: &demes::Graph) {
    // Enumerate the &[demes::Pulse]
    for (i, pulse) in graph.pulses().iter().enumerate() {
        println!("pulse {i}");
        println!("\ttime: {}", pulse.time());

        // Slices don't implement Display, so we
        // format things manually
        print!("\tsources: [");
        for s in pulse.sources() {
            print!("{s}, ");
        }
        println!("]");
        println!("\tdest: {}", pulse.dest());
        print!("\tproportions: [");
        for p in pulse.proportions() {
            print!("{p}, ");
        }
        println!("]");
    }
}

fn do_work(path: &str) -> Result<()> {
    let graph = load_yaml(path)?;

    // demes::Graph implements Display, which
    // writes out the YAML representation
    // of the fully-resolved graph.
    // Note: the implementation of Display
    // wraps a call to graph.as_string().unwrap(),
    // which returns a YAML string. The as_string
    // function may return an error from serde_yaml,
    // although that is very unlikely to happen
    // for a resolved graph.
    display_graph(&graph);

    // If demes is build with the json feature,
    // then we can get a JSON string representation
    // of the graph.
    // Note that the #[cfg(...)] business is only needed
    // here because this file is part of the demes-rs repo.
    // See the section in the book on features:
    // https://doc.rust-lang.org/cargo/reference/features.html
    #[cfg(feature = "json")]
    {
        println!(
            "The graph in JSON format:\n{}",
            graph.as_json_string().unwrap()
        );
    }

    graph_api_examples(&graph);

    iterate_demes_and_epochs(&graph);

    iterate_migrations(&graph);

    iterate_pulses(&graph);

    Ok(())
}

fn main() {
    do_work("examples/jouganous.yaml").unwrap();
}

#[test]
fn test_jouganous_model() {
    do_work("examples/jouganous.yaml").unwrap();
}
