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
    field5: InnerParams0,
    /// Field 6
    field6: Enum,
    /// Field 7
    field7: Vec<i16>,
    /// Field 8
    field8: Vec<i16>,
}

#[derive(grib_template_derive::Dump)]
#[repr(u8)]
pub enum Enum {
    /// Field 1
    Type0(InnerParams0) = 0,
    /// Field 2
    Type1(InnerParams1) = 1,
}

#[derive(grib_template_derive::Dump)]
pub struct InnerParams0 {
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

#[derive(Debug, PartialEq, Eq, grib_template_derive::Dump)]
pub struct InnerParams1 {
    /// Field 1
    field1: u8,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::Dump)]
pub struct ParamsWithGenerics<T: grib_template_helpers::DumpField> {
    /// Field 1
    field1: T,
}

pub type TypeWithGenerics = ParamsWithGenerics<i16>;

fn main() {}
