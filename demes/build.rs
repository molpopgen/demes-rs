use std::env;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    build_valid_spec_examples_tests();
    build_invalid_spec_examples_tests();
}

fn build_valid_spec_examples_tests() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("valid_specification_tests.rs");
    let mut test_file = File::create(&destination).unwrap();
    let paths = read_dir("demes-spec/test-cases/valid").unwrap();
    for p in paths {
        let p = p.unwrap();
        write_valid_example_test(&mut test_file, &p);
    }
}

fn write_valid_example_test(test_file: &mut File, path: &DirEntry) {
    let directory = path.path().canonicalize().unwrap();
    let test_name = directory.file_name().unwrap().to_string_lossy();
    let full_path = directory.to_string_lossy();
    let test_name = test_name.replace(".yaml", "");
    let test_name = format!("test_valid_case_{}", test_name);
    write!(
        test_file,
        include_str!("./tests/valid_test_template"),
        name = test_name,
        path = full_path,
    )
    .unwrap();
}

fn build_invalid_spec_examples_tests() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("invalid_specification_tests.rs");
    let mut test_file = File::create(&destination).unwrap();
    let paths = read_dir("demes-spec/test-cases/invalid").unwrap();
    for p in paths {
        let p = p.unwrap();
        write_invalid_example_test(&mut test_file, &p);
    }
}

fn write_invalid_example_test(test_file: &mut File, path: &DirEntry) {
    let directory = path.path().canonicalize().unwrap();
    let test_name = directory.file_name().unwrap().to_string_lossy();
    let full_path = directory.to_string_lossy();
    let test_name = test_name.replace(".yaml", "");
    let test_name = format!("test_invalid_case_{}", test_name);
    write!(
        test_file,
        include_str!("./tests/invalid_test_template"),
        name = test_name,
        path = full_path,
    )
    .unwrap();
}
