#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/dumping.rs");
    t.pass("tests/reading.rs");
}
