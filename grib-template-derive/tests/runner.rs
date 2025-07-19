#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/dump.rs");
    t.pass("tests/try_from_slice.rs");
}
