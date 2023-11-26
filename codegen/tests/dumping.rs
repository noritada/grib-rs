use grib_shared::Dump;

#[derive(grib_codegen::Dump)]
pub struct Params {
    // #[doc = "Field 1"]
    field1: u8,
    // #[doc = "Field 2"]
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
        "field1: 1\nfield2: 2\nfield3: 3\nfield4: 4\n"
    )
}
