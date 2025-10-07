use grib_template_helpers::Dump;

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

fn main() {
    let params = Params {
        field1: 1,
        field2: 2,
        field3: 3,
        field4: 4.0,
        field5: InnerParams0 {
            field1: 51,
            field2: 52,
            field3: InnerInnerParams { field1: 511 },
        },
        field6: Enum::Type1(InnerParams1 { field1: 61 }),
        field7: vec![1, 2, 3, 4],
        field8: vec![1],
    };

    let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
    let mut pos = 1;
    params.dump(None, &mut pos, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8_lossy(buf.get_ref()),
        "\
1         field1 = 1  // Field 1
2-3       field2 = 2  // Field 2
4-5       field3 = 3  // Field 3
6-9       field4 = 4.0  // Field 4
10        field5.field1 = 51  // Field 5.1
11        field5.field2 = 52  // Field 5.2
12-13     field5.field3.field1 = 511  // Field 5.3.1
14        field6.field1 = 61  // Field 1
15-22     field7 = [1, 2, 3, 4]  // Field 7
23-24     field8 = [1]  // Field 8
"
    )
}
