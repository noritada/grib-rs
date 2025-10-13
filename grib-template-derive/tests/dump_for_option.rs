use grib_template_helpers::Dump;

#[derive(Debug, PartialEq, grib_template_derive::Dump)]
pub struct OptionalStruct {
    field1: u8,
    field2: Option<u16>,
    field3: Option<Struct>,
}

#[derive(Debug, PartialEq, grib_template_derive::Dump)]
pub struct Struct {
    field1: i16,
}

#[derive(Debug, PartialEq, grib_template_derive::Dump)]
pub struct OptionalVec {
    field1: u8,
    field2: Option<u16>,
    field3: Option<Vec<i16>>,
}

#[derive(Debug, PartialEq, grib_template_derive::Dump)]
pub struct OptionalEnum {
    field1: u8,
    field2: Option<u8>,
    field3: Option<Enum>,
}

#[derive(Debug, PartialEq, grib_template_derive::Dump)]
#[repr(u8)]
pub enum Enum {
    Var1(EnumVar1) = 1,
    Var2(EnumVar2) = 2,
}

#[derive(Debug, PartialEq, grib_template_derive::Dump)]
pub struct EnumVar1 {
    field1: u16,
}

#[derive(Debug, PartialEq, grib_template_derive::Dump)]
pub struct EnumVar2 {
    field1: i16,
}

macro_rules! test {
    ($((
        $input:expr,
        $expected:expr,
    ),)*) => ($(
        let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
        let mut pos = 1;
        $input.dump(None, &mut pos, &mut buf).unwrap();
        assert_eq!(String::from_utf8_lossy(buf.get_ref()), $expected);
    )*);
}

fn main() {
    test![
        (
            OptionalStruct {
                field1: 2,
                field2: Some(2),
                field3: Some(Struct { field1: -1 }),
            },
            "\
1         field1 = 2
2-3       field2 = 2
4-5       field3.field1 = -1
",
        ),
        (
            OptionalStruct {
                field1: 2,
                field2: None,
                field3: None,
            },
            "\
1         field1 = 2
",
        ),
        (
            OptionalVec {
                field1: 2,
                field2: Some(2),
                field3: Some(vec![-1, -2]),
            },
            "\
1         field1 = 2
2-3       field2 = 2
4-7       field3 = [-1, -2]
",
        ),
        (
            OptionalVec {
                field1: 2,
                field2: None,
                field3: None,
            },
            "\
1         field1 = 2
",
        ),
        (
            OptionalEnum {
                field1: 2,
                field2: Some(2),
                field3: Some(Enum::Var2(EnumVar2 { field1: -0x0002 })),
            },
            "\
1         field1 = 2
2         field2 = 2
3-4       field3.field1 = -2
",
        ),
        (
            OptionalEnum {
                field1: 2,
                field2: None,
                field3: None,
            },
            "\
1         field1 = 2
",
        ),
    ];
}
