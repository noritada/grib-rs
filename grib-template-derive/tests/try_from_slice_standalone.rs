#![allow(dead_code)]

#[derive(Debug, PartialEq, Eq, grib_template_derive::TryFromSlice)]
pub struct Params {
    /// Field 1
    field1: u8,
    /// Field 2
    field2: u16,
    /// Field 3
    field3: i16,
    /// Field 4
    field4: u32,
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

fn main() {}
