#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-normal-case.rs");
    t.compile_fail("tests/02-not-empty-enum.rs");
    t.compile_fail("tests/03-nonexistent-file.rs");
}
