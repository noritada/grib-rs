use grib_template_helpers::TryFromSlice;

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct OptionalStruct {
    field1: u8,
    field2: Option<u16>,
    field3: Option<Struct>,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct Struct {
    field1: i16,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct OptionalVec {
    field1: u8,
    field2: Option<u16>,
    #[grib_template(len = "field1")]
    field3: Option<Vec<i16>>,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct OptionalEnum {
    field1: u8,
    field2: Option<u8>,
    #[grib_template(variant = "field1")]
    field3: Option<Enum>,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
#[repr(u8)]
pub enum Enum {
    Var1(EnumVar1) = 1,
    Var2(EnumVar2) = 2,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct EnumVar1 {
    field1: u16,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct EnumVar2 {
    field1: i16,
}

macro_rules! test {
    ($((
        $buf:expr,
        $ty:ident,
        $expected:expr,
    ),)*) => ($(
        let buf: [u8; _] = $buf;
        let mut pos = 0;
        let actual = $ty::try_from_slice(&buf, &mut pos);
        let expected = $expected;
        assert_eq!(actual, expected);
    )*);
}

fn main() {
    test![
        (
            [0x02, 0x00, 0x02, 0x80, 0x01],
            OptionalStruct,
            Ok(OptionalStruct {
                field1: 2,
                field2: Some(2),
                field3: Some(Struct { field1: -1 }),
            }),
        ),
        (
            [0x02],
            OptionalStruct,
            Ok(OptionalStruct {
                field1: 2,
                field2: None,
                field3: None,
            }),
        ),
        (
            [0x02, 0x00],
            OptionalStruct,
            Err("slice length is too short"),
        ),
        (
            [0x02, 0x00, 0x02, 0x80],
            OptionalStruct,
            Err("slice length is too short"),
        ),
        (
            [0x02, 0x00, 0x02, 0x80, 0x01, 0x80, 0x02],
            OptionalVec,
            Ok(OptionalVec {
                field1: 2,
                field2: Some(2),
                field3: Some(vec![-1, -2]),
            }),
        ),
        (
            [0x02],
            OptionalVec,
            Ok(OptionalVec {
                field1: 2,
                field2: None,
                field3: None,
            }),
        ),
        (
            [0x02, 0x00, 0x02, 0x80, 0x01],
            OptionalVec,
            Err("slice length is too short"),
        ),
        (
            [0x02, 0x02, 0x80, 0x02],
            OptionalEnum,
            Ok(OptionalEnum {
                field1: 2,
                field2: Some(2),
                field3: Some(Enum::Var2(EnumVar2 { field1: -0x0002 })),
            }),
        ),
        (
            [0x02],
            OptionalEnum,
            Ok(OptionalEnum {
                field1: 2,
                field2: None,
                field3: None,
            }),
        ),
        (
            [0x02, 0x02, 0x80],
            OptionalEnum,
            Err("slice length is too short"),
        ),
    ];
}
