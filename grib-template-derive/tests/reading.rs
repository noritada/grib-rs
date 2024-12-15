use grib_data_helpers::FromSlice;

#[derive(Debug, PartialEq, Eq, grib_template_derive::FromSlice)]
pub struct Params {
    /// Field 1
    field1: u8,
    /// Field 2
    field2: u16,
    /// Field 3
    field3: u16,
    /// Field 4
    field4: u32,
}

fn main() {
    let buf = vec![0x01_u8, 0xff, 0x00, 0x01, 0x23, 0x76, 0x54, 0x32, 0x10];
    let actual = Params::from_slice(&buf);
    let expected = Params {
        field1: 0x01,
        field2: 0xff00,
        field3: 0x0123,
        field4: 0x76543210,
    };

    assert_eq!(actual, expected)
}
