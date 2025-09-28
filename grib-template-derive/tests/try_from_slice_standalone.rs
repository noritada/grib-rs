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
    field5: InnerParams0,
    /// Field 6
    #[grib_template(variant = "field1")]
    field6: Enum,
    /// Field 7
    #[grib_template(len = 4)]
    field7: Vec<i16>,
    /// Field 8
    #[grib_template(len = "field1")]
    field8: Vec<i16>,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::TryFromSlice)]
#[repr(u8)]
pub enum Enum {
    /// Field 1
    Type0(InnerParams0) = 0,
    /// Field 2
    Type1(InnerParams1) = 1,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::TryFromSlice)]
pub struct InnerParams0 {
    /// Field 1
    field1: u8,
    /// Field 2
    field2: u8,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::TryFromSlice)]
pub struct InnerParams1 {
    /// Field 1
    field1: u8,
}

fn main() {}
