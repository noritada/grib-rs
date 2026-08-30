use grib_template_helpers::TryFromSlice;

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct StructWithExhaustiveEnum {
    field1: u8,
    #[grib_template(variant = "field1")]
    field2: ExhaustiveEnum,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
pub struct StructWithNonExhaustiveEnum {
    field1: u8,
    #[grib_template(variant = "field1")]
    field2: NonExhaustiveEnum,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
#[repr(u8)]
pub enum ExhaustiveEnum {
    Var1(EnumVar1) = 1,
    Var2(EnumVar2) = 2,
}

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice)]
#[non_exhaustive]
#[repr(u8)]
pub enum NonExhaustiveEnum {
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
            [0x02, 0x80, 0x02],
            StructWithExhaustiveEnum,
            Ok(StructWithExhaustiveEnum {
                field1: 2,
                field2: ExhaustiveEnum::Var2(EnumVar2 { field1: -0x0002 }),
            }),
        ),
        (
            [0x02, 0x80, 0x02],
            StructWithNonExhaustiveEnum,
            Ok(StructWithNonExhaustiveEnum {
                field1: 2,
                field2: NonExhaustiveEnum::Var2(EnumVar2 { field1: -0x0002 }),
            }),
        ),
    ];
}
