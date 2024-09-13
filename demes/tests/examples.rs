#[test]
fn test_doi() {
    let file = std::fs::File::open("demes-spec/test-cases/valid/jacobs_papuans.yaml").unwrap();
    let graph = demes::load(file).unwrap();
    assert_eq!(graph.doi().count(), 2);
    let expected = [
        "https://doi.org/10.1016/j.cell.2019.02.035".to_owned(),
        "https://doi.org/10.1038/nature18299".to_owned(),
    ];
    for (i, j) in graph.doi().zip(expected.iter()) {
        assert_eq!(i, j)
    }
}

#[test]
fn test_graph_with_description() {
    let yaml = "
description: A great demes graph
time_units: generations
demes:
 - name: demeA
   epochs:
    - start_size: 100
";
    let graph = demes::loads(yaml).unwrap();
    assert_eq!(graph.description(), Some("A great demes graph"));
}

#[test]
fn test_graph_without_description() {
    let yaml = "
time_units: generations
demes:
 - name: demeA
   epochs:
    - start_size: 100
";
    let graph = demes::loads(yaml).unwrap();
    assert!(graph.description().is_none());
}
