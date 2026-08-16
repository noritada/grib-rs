use grib_template_helpers::Dump;

#[derive(grib_template_derive::Dump)]
pub struct Params {
    /// Field 1
    field1: ReusableType,
    #[dump(doc(
        element_id = "Id of the second element",
        factor = "Scale factor of the second element",
    ))]
    /// Field 2
    field2: ReusableType,
}

#[derive(grib_template_derive::Dump)]
pub struct ReusableType {
    /// Id
    element_id: u8,
    /// Scale factor
    factor: u8,
    /// Scaled value
    value: u32,
}

fn main() {
    let params = Params {
        field1: ReusableType {
            element_id: 1,
            factor: 1,
            value: 1,
        },
        field2: ReusableType {
            element_id: 2,
            factor: 2,
            value: 2,
        },
    };

    let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
    let mut pos = 1;
    params.dump(None, None, &mut pos, &mut buf).unwrap();
    assert_eq!(
        String::from_utf8_lossy(buf.get_ref()),
        "\
1         field1.element_id = 1  // Id
2         field1.factor = 1  // Scale factor
3-6       field1.value = 1  // Scaled value
7         field2.element_id = 2  // Id of the second element
8         field2.factor = 2  // Scale factor of the second element
9-12      field2.value = 2  // Scaled value
"
    )
}
