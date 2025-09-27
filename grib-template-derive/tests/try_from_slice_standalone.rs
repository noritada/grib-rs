#![allow(dead_code)]

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
    /// Field 6
    #[try_from_slice(len = 4)]
    field6: Vec<u16>,
    /// Field 7
    #[try_from_slice(len = "field1")]
    field7: Vec<u16>,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::TryFromSlice)]
pub struct InnerParams {
    /// Field 1
    field1: u8,
    /// Field 2
    field2: u8,
}

fn main() {}
