use grib_template_helpers::{DocOverrides, Dump, TryFromSlice, WriteToBuffer};

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    grib_template_derive::TryFromSlice,
    grib_template_derive::WriteToBuffer,
    grib_template_derive::Dump,
)]
struct TemplateNumber(
    /// Template number
    u16,
);

impl From<TemplateNumber> for u64 {
    fn from(value: TemplateNumber) -> Self {
        u64::from(value.0)
    }
}

#[derive(
    Debug, PartialEq, grib_template_derive::TryFromSlice, grib_template_derive::WriteToBuffer,
)]
struct Header {
    template_num: TemplateNumber,
    #[grib_template(variant = "template_num")]
    template: Template,
    trailing: u8,
}

#[derive(
    Debug, PartialEq, grib_template_derive::TryFromSlice, grib_template_derive::WriteToBuffer,
)]
#[repr(u16)]
enum Template {
    First(u8) = 0,
    Local(u8) = 50000,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = [0xc3, 0x50, 7, 9];
    let mut pos = 0;
    let value = Header::try_from_slice(&bytes, &mut pos)?;
    assert_eq!(
        value,
        Header {
            template_num: TemplateNumber(50000),
            template: Template::Local(7),
            trailing: 9,
        }
    );
    assert_eq!(pos, bytes.len());
    let mut output = [0; 4];
    assert_eq!(value.num_bytes_required(), 4);
    assert_eq!(value.write_to_buffer(&mut output)?, 4);
    assert_eq!(output, bytes);
    assert!(value.write_to_buffer(&mut [0; 3]).is_err());

    let mut output = Vec::new();
    let mut pos = 1;
    // Dump the wrapper and then a named field to verify octet advancement.
    value.template_num.dump(
        Some(&"template_num".into()),
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    grib_template_helpers::DumpField::dump_field(
        &value.trailing,
        "trailing",
        None,
        None,
        DocOverrides::empty(),
        &mut pos,
        &mut output,
    )?;
    assert_eq!(
        String::from_utf8(output)?,
        "1-2       template_num = 50000  // Template number\n3         trailing = 9\n"
    );
    assert_eq!(pos, 4);

    let mut pos = 0;
    assert_eq!(
        Header::try_from_slice(&[0, 1, 7, 9], &mut pos),
        Err("unknown enum discriminant")
    );
    assert_eq!(pos, 2);
    assert!(Header::try_from_slice(&[0xc3], &mut 0).is_err());
    let mut pos = 0;
    assert_eq!(
        Header::try_from_slice(&[0, 0, 7, 9], &mut pos)?.template,
        Template::First(7)
    );
    Ok(())
}
