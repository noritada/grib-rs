use grib_template_helpers::WriteToBuffer;

#[derive(Debug, PartialEq, grib_template_derive::WriteToBuffer)]
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
    /// Field 7
    field7: Vec<i16>,
    /// Field 8
    field8: Vec<i16>,
    field9: TypeWithGenerics,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::WriteToBuffer)]
pub struct InnerParams0 {
    /// Field 1
    field1: u8,
    /// Field 2
    field2: u8,
}

#[derive(Debug, PartialEq, Eq, grib_template_derive::WriteToBuffer)]
pub struct ParamsWithGenerics<T: grib_template_helpers::WriteToBuffer> {
    /// Field 1
    field1: T,
}

pub type TypeWithGenerics = ParamsWithGenerics<i16>;

fn main() {
    let params = Params {
        field1: 0x01,
        field2: 0xff00,
        field3: -0x7f00,
        field4: 1.0,
        field5: InnerParams0 {
            field1: 0xf0,
            field2: 0x0f,
        },
        field7: vec![-0x70f1, -0x72f3, -0x74f5, -0x76f7],
        field8: vec![-0x70f1],
        field9: ParamsWithGenerics { field1: -0x70f1 },
    };
    let mut buf = vec![0_u8; 24];
    let result = params.write_to_buffer(&mut buf);
    assert_eq!(result, Ok(23));
    let expected_buf = vec![
        0x01_u8, 0xff, 0x00, 0xff, 0x00, 0x3f, 0x80, 0x00, 0x00, 0xf0, 0x0f, 0xf0, 0xf1, 0xf2,
        0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf0, 0xf1, 0xf0, 0xf1, 0,
    ];
    assert_eq!(buf, expected_buf);
}
