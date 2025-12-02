use std::fs;
use std::path::Path;

#[test]
fn compile_tests() {
    let dirpath = Path::new("tests/compile_errors");
    assert!(dirpath.is_dir(), "compile test folder does not exists");

    let subelts = fs::read_dir(dirpath).expect("failed to list test folder content");

    // see https://docs.rs/trybuild/latest/trybuild/
    let tests = trybuild::TestCases::new();
    tests.compile_fail(dirpath.join("*.rs"));
    for elt in subelts.flatten() {
        let path = elt.path();
        if path.is_dir() {
            tests.compile_fail(path.join("*.rs"));
        }
    }
}
