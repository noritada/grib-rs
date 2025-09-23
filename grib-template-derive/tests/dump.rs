use grib_data_helpers::Dump;

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
}

fn main() {
    let params = Params {
        field1: 1,
        field2: 2,
        field3: 3,
        field4: 4.0,
    };

    let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
    params.dump(&mut buf).unwrap();
    assert_eq!(
        String::from_utf8_lossy(buf.get_ref()),
        "\
1: field1 = 1  // Field 1
2-3: field2 = 2  // Field 2
4-5: field3 = 3  // Field 3
6-9: field4 = 4  // Field 4
"
    )
}
