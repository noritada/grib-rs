#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-normal-case.rs");
}
