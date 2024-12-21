#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/dump.rs");
    t.pass("tests/from_slice.rs");
}
