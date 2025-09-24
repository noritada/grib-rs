#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/dump.rs");
    t.pass("tests/dump_standalone.rs");
    t.pass("tests/try_from_slice.rs");
    t.pass("tests/try_from_slice_standalone.rs");
}
