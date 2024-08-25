use demes::Graph;

fn validate_names_in_graph(graph: &Graph) {
    let names: Vec<String> = graph.demes().iter().map(|d| d.name().to_owned()).collect();
    for (i, &n) in graph.deme_names().iter().enumerate() {
        assert_eq!(&names[i], n);
        if let Some(deme) = graph.get_deme(i) {
            assert_eq!(n, deme.name())
        } else {
            panic!();
        }
        if let Some(deme) = graph.get_deme(n) {
            assert_eq!(n, deme.name())
        } else {
            panic!();
        }
    }
}

fn round_trip_equality(graph: &Graph) {
    let s = graph.as_string().unwrap();
    let round_trip = demes::loads(&s).unwrap();
    assert_eq!(graph, &round_trip);
}

#[cfg(feature = "json")]
fn json_roundtrip(graph: &Graph, filename: &str) {
    use std::io::Read;

    let mut f = std::fs::File::open(filename).unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf).unwrap();
    let json = serde_yaml::from_str::<serde_json::Value>(&buf).unwrap();
    let json = json.to_string();
    let graph_from_json = demes::loads_json(&json).unwrap();
    assert_eq!(graph, &graph_from_json, "{filename:?}");

    // Read from a type implementing Read
    let raw_bytes = json.as_bytes();
    let graph_from_raw_bytes = demes::load_json(raw_bytes).unwrap();
    assert_eq!(graph, &graph_from_raw_bytes);
}

#[cfg(feature = "toml")]
fn filter_yaml_input(
    input: std::collections::HashMap<String, serde_yaml::Value>,
) -> (std::collections::HashMap<String, serde_yaml::Value>, bool) {
    let mut input = input;

    if let Some(serde_yaml::Value::Mapping(metadata)) = input.get_mut("metadata") {
        let mut keys = vec![];
        for (k, v) in metadata.iter() {
            if let serde_yaml::Value::Null = v {
                keys.push(k.clone());
            }
        }
        for key in keys {
            metadata.remove(&key);
        }
        (input, true)
    } else {
        (input, false)
    }
}

#[cfg(feature = "toml")]
fn toml_roundtrip(graph: &Graph, filename: &str) {
    use std::io::Read;

    let mut f = std::fs::File::open(filename).unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf).unwrap();
    let yaml: std::collections::HashMap<String, serde_yaml::Value> =
        serde_yaml::from_str(&buf).unwrap();
    let (yaml, yaml_was_modified) = filter_yaml_input(yaml);
    let yaml = serde_yaml::to_string(&yaml).unwrap();
    let toml = serde_yaml::from_str::<toml::Value>(&yaml).unwrap();
    let toml = toml::to_string(&toml).unwrap();
    let graph_from_toml = demes::loads_toml(&toml).unwrap();
    if !yaml_was_modified {
        assert_eq!(graph, &graph_from_toml, "{filename:?}");
    } else {
        // The metadata have been modified, so we compare all other elements
        assert_eq!(graph.demes(), graph_from_toml.demes());
        assert_eq!(graph.migrations(), graph_from_toml.migrations());
        assert_eq!(graph.pulses(), graph_from_toml.pulses());
        assert_eq!(graph.time_units(), graph_from_toml.time_units());
        assert_eq!(graph.description(), graph_from_toml.description());
        assert_eq!(
            graph.doi().collect::<Vec<&str>>(),
            graph_from_toml.doi().collect::<Vec<&str>>()
        );
    }
    //// Read from a type implementing Read
    let raw_bytes = toml.as_bytes();
    let graph_from_raw_bytes = demes::load_toml(raw_bytes).unwrap();
    if !yaml_was_modified {
        assert_eq!(graph, &graph_from_raw_bytes);
    } else {
        assert_eq!(graph_from_raw_bytes, graph_from_toml);
    }
}

// NOTE: these test cases are automatically build in build.rs
include!(concat!(env!("OUT_DIR"), "/valid_specification_tests.rs"));
include!(concat!(env!("OUT_DIR"), "/invalid_specification_tests.rs"));
