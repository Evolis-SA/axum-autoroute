// Build error messages are dependent on rust version.
// Disable these tests if we're not on stable.
#[rustversion::since(1.91)]
#[test]
fn compile_tests() {
    let dirpath = std::path::Path::new("tests/compile_errors");
    assert!(dirpath.is_dir(), "compile test folder does not exists");

    let subelts = std::fs::read_dir(dirpath).expect("failed to list test folder content");

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
