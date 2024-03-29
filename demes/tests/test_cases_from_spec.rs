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

#[cfg(not(feature = "json"))]
fn json_roundtrip(_: Graph, _: &str) {}

#[cfg(feature = "json")]
fn json_roundtrip(graph: Graph, filename: &str) {
    let json = graph.as_json_string().unwrap();
    let graph_from_json = demes::loads_json(&json).unwrap();
    assert_eq!(graph, graph_from_json, "{filename:?}");

    // Read from a type implementing Read
    let raw_bytes = json.as_bytes();
    let graph_from_raw_bytes = demes::load_json(raw_bytes).unwrap();
    assert_eq!(graph, graph_from_raw_bytes);
}

// NOTE: these test cases are automatically build in build.rs
include!(concat!(env!("OUT_DIR"), "/valid_specification_tests.rs"));
include!(concat!(env!("OUT_DIR"), "/invalid_specification_tests.rs"));
