use grib_template_helpers::{CodeTable, DocOverrides, Dump};

use crate::{
    Grib2Context,
    def::grib2::table::{CentreId, MasterTableVer, SubcentreId},
};

macro_rules! test_lookup {
    ($($name:ident: $value:expr => $expected:expr),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                let context = Grib2Context { discipline: 0 };
                assert_eq!($value.code_description(&context), $expected);
                assert_eq!($value.code_unit(&context), None);
            }
        )*
    };
}

test_lookup! {
    centre: CentreId(34) => Some("Tokyo (RSMC), Japan Meteorological Agency"),
    reserved_centre: CentreId(65534) => None,
    missing_centre: CentreId(65535) => None,
    experimental_version: MasterTableVer(0) => Some("Experimental"),
    effective_date: MasterTableVer(5) => Some("4 November 2009"),
    future_version: MasterTableVer(254) => None,
    missing_version: MasterTableVer(255) => None,
}

#[test]
fn placeholders_and_subcentre_dump_without_descriptions() -> Result<(), Box<dyn std::error::Error>>
{
    let mut output = Vec::new();
    let mut pos = 1;
    CentreId(65535).dump(
        &(),
        Some(&"centre".into()),
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    SubcentreId(0).dump(
        &(),
        Some(&"subcentre".into()),
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    MasterTableVer(255).dump(
        &(),
        Some(&"version".into()),
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    assert_eq!(
        String::from_utf8(output)?,
        "1-2       centre = 65535  // Identification of originating/generating centre (see Common Code table C-11).\n3-4       subcentre = 0  // Identification of originating/generating subcentre (allocated by originating/generating centre).\n5         version = 255  // GRIB master table version number (see Common Code table C-0 and Note 1).\n",
    );
    assert_eq!(pos, 6);
    Ok(())
}
