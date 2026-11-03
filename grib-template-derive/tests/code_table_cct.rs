use grib_template_helpers::{CodeTable, DocOverrides, Dump};

#[derive(grib_template_derive::CodeTable, grib_template_derive::Dump)]
#[code_table(file = "tests/tables/cct_c00.csv", format = wmo_cct_c00)]
#[dump(code_table)]
struct Version(u8);

#[derive(grib_template_derive::CodeTable, grib_template_derive::Dump)]
#[code_table(file = "tests/tables/cct_c11.csv", format = wmo_cct_c11)]
#[dump(code_table)]
struct Centre(u16);

macro_rules! test_lookup {
    ($($name:ident: $value:expr => $expected:expr),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                assert_eq!($value.code_description(&()), $expected);
                assert_eq!($value.code_unit(&()), None);
            }
        )*
    };
}

test_lookup! {
    experimental: Version(0) => Some("Experimental"),
    effective_date: Version(1) => Some("1 January 2000"),
    future: Version(2) => None,
    missing_version: Version(255) => None,
    centre: Centre(0) => Some("WMO Secretariat"),
    quoted: Centre(1) => Some("Example, with \"quotes\""),
    range_start: Centre(2) => Some("Example centres"),
    range_middle: Centre(3) => Some("Example centres"),
    range_end: Centre(4) => Some("Example centres"),
    reserved: Centre(5) => None,
    missing_centre: Centre(65535) => None,
}

#[test]
fn dumps_common_code_descriptions() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = Vec::new();
    let mut pos = 1;
    Version(1).dump(
        &(),
        Some(&"version".into()),
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    Centre(0).dump(
        &(),
        Some(&"centre".into()),
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    assert_eq!(
        String::from_utf8(output)?,
        "1         version = 1 [1 January 2000]\n2-3       centre = 0 [WMO Secretariat]\n"
    );
    assert_eq!(pos, 4);
    Ok(())
}
