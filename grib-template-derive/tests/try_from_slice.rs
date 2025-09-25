use grib_data_helpers::TryFromSlice;

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct Params {
    /// Field 1
    field1: u8,
    /// Field 2
    field2: u16,
    /// Field 3
    field3: i16,
    /// Field 4
    field4: f32,
    /// Field 5
    field5: InnerParams,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::TryFromSlice)]
pub struct InnerParams {
    /// Field 1
    field1: u8,
    /// Field 2
    field2: u8,
}

fn main() {
    let buf = vec![
        0x01_u8, 0xff, 0x00, 0xff, 0x00, 0x3f, 0x80, 0x00, 0x00, 0xf0, 0x0f,
    ];
    let actual = Params::try_from_slice(&buf);
    let expected = Ok(Params {
        field1: 0x01,
        field2: 0xff00,
        field3: -0x7f00,
        field4: 1.0,
        field5: InnerParams {
            field1: 0xf0,
            field2: 0x0f,
        },
    });

    assert_eq!(actual, expected)
}
