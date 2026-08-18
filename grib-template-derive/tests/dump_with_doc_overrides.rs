use grib_template_helpers::Dump;

#[derive(grib_template_derive::Dump)]
pub struct Params {
    /// Field 1
    field1: ReusableType,
    #[dump(doc(field1 = "Field 2.1", field2(element_id = "Super-overrided id",),))]
    /// Field 2
    field2: ReusableComponent,
    #[dump(doc(
        element_id = "Id of the third element",
        factor = "Scale factor of the third element",
    ))]
    /// Field 3
    field3: ReusableType,
}

#[derive(grib_template_derive::Dump)]
pub struct ReusableComponent {
    /// Component field 1
    field1: u8,
    #[dump(doc(element_id = "Overrided id", factor = "Overrided scale factor",))]
    /// Component field 2
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
        field2: ReusableComponent {
            field1: 2,
            field2: ReusableType {
                element_id: 2,
                factor: 2,
                value: 2,
            },
        },
        field3: ReusableType {
            element_id: 3,
            factor: 3,
            value: 3,
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
7         field2.field1 = 2  // Field 2.1
8         field2.field2.element_id = 2  // Super-overrided id
9         field2.field2.factor = 2  // Overrided scale factor
10-13     field2.field2.value = 2  // Scaled value
14        field3.element_id = 3  // Id of the third element
15        field3.factor = 3  // Scale factor of the third element
16-19     field3.value = 3  // Scaled value
"
    )
}
