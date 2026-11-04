use grib_template_helpers::{DocOverrides, Dump, MissingValue};

#[derive(grib_template_derive::Dump)]
struct Values {
    unsigned: u8,
    signed: i8,
    float: f32,
    double: f64,
    #[grib_template(num_octets = 3)]
    short: u32,
    values: Vec<u8>,
    bytes: [u8; 2],
    flag: Flag,
    scalar: Scalar,
}

#[derive(grib_template_derive::Dump)]
#[dump(flag)]
struct Flag(u8);

#[derive(grib_template_derive::Dump)]
struct Scalar(u8);

#[test]
fn missing_scalars_leave_collections_and_flags_unchanged() -> Result<(), Box<dyn std::error::Error>>
{
    let values = Values {
        unsigned: u8::missing(),
        signed: i8::missing(),
        float: f32::missing(),
        double: f64::missing(),
        short: 0xffffff,
        values: vec![255, 1],
        bytes: [255, 255],
        flag: Flag(255),
        scalar: Scalar(255),
    };
    let mut output = Vec::new();
    let mut pos = 1;
    values.dump(&(), None, DocOverrides::empty(), &mut pos, &mut output)?;
    assert_eq!(
        String::from_utf8(output)?,
        "1         unsigned = 255 [Missing]
2         signed = -127 [Missing]
3-6       float = NaN [Missing]
7-14      double = NaN [Missing]
15-17     short = 16777215 [Missing]
18-19     values = [255, 1]
20-21     bytes = [255, 255]
22        flag = 0b11111111
23        scalar = 255 [Missing]
"
    );
    assert_eq!(pos, 24);
    Ok(())
}
