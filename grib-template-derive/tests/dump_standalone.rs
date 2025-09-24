#![allow(dead_code)]

#[derive(grib_template_derive::Dump)]
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

#[derive(grib_template_derive::Dump)]
pub struct InnerParams {
    /// Field 5.1
    field1: u8,
    /// Field 5.2
    field2: u8,
    /// Field 5.3
    field3: InnerInnerParams,
}

#[derive(grib_template_derive::Dump)]
pub struct InnerInnerParams {
    /// Field 5.3.1
    field1: u16,
}

fn main() {}
