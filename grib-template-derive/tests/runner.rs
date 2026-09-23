use grib_template_helpers::TryFromSlice;

#[derive(grib_template_derive::TryFromSlice)]
#[allow(dead_code)]
struct TemplateHeader {
    template_num: u8,
    #[grib_template(variant = "template_num")]
    template: Template,
}

#[derive(grib_template_derive::TryFromSlice)]
#[allow(dead_code)]
#[repr(u8)]
enum Template {
    Known(u8) = 0,
}

#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/dump.rs");
    t.pass("tests/dump_standalone.rs");
    t.pass("tests/dump_for_option.rs");
    t.pass("tests/dump_with_doc_overrides.rs");
    t.pass("tests/try_from_slice.rs");
    t.pass("tests/try_from_slice_standalone.rs");
    t.pass("tests/try_from_slice_for_enum.rs");
    t.pass("tests/try_from_slice_for_option.rs");
    t.pass("tests/write_to_buffer.rs");
}

#[test]
fn unknown_template_discriminant_returns_error() {
    let mut pos = 0;
    assert!(matches!(
        TemplateHeader::try_from_slice(&[1], &mut pos),
        Err("unknown enum discriminant"),
    ));
}
