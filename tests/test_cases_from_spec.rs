use glob::glob;

fn invalid_file_skip_list() -> std::collections::HashSet<String> {
    let mut rv = std::collections::HashSet::default();

    rv.insert("demes-spec/test-cases/invalid/bad_demelevel_defaults_epoch_43.yaml".to_string());
    rv.insert("demes-spec/test-cases/invalid/bad_toplevel_defaults_epoch_43.yaml".to_string());
    rv.insert("demes-spec/test-cases/invalid/bad_size_function_04.yaml".to_string());

    rv
}

fn process_path(
    path: &str,
    skip_list: Option<std::collections::HashSet<String>>,
) -> (Vec<String>, Vec<String>) {
    let paths = glob(path).unwrap();
    let mut failures = vec![];
    let mut successes = vec![];
    for path in paths {
        let name = path.unwrap();
        let should_skip = match skip_list.as_ref() {
            None => false,
            Some(sl) => sl.contains(&name.clone().to_str().unwrap().to_string()),
        };

        if !should_skip {
            let file = std::fs::File::open(name.clone()).unwrap();
            match demes::load(file) {
                Ok(_) => successes.push(name.to_str().unwrap().to_owned()),
                Err(_) => failures.push(name.to_str().unwrap().to_owned()),
            }
        }
    }
    (successes, failures)
}

#[test]
fn load_valid_graphs() {
    let rv = process_path("demes-spec/test-cases/valid/*.yaml", None);
    assert!(rv.1.is_empty(), "{:?}", rv.1);
}

#[test]
fn load_invalid_graphs() {
    let skip_list = Some(invalid_file_skip_list());
    let rv = process_path("demes-spec/test-cases/invalid/*.yaml", skip_list);
    assert!(rv.0.is_empty(), "{:?}", rv.0);
}
