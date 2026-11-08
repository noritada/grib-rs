#![allow(dead_code)]

use grib_template_helpers::{FieldOffset, FixedSize, ReadFixedField, TryFromSlice};

#[derive(Debug, PartialEq, grib_template_derive::TryFromSlice, grib_template_derive::FixedSize)]
struct Param {
    category: u8,
    number: u16,
}

#[derive(grib_template_derive::FieldOffset, grib_template_derive::TryFromSlice)]
struct TemplateA {
    prefix: u8,
    #[grib_template(num_octets = 3)]
    packed: u32,
    #[field_offset(Param)]
    param: Param,
    count: u8,
    #[grib_template(len = "count")]
    tail: Vec<u8>,
}

#[derive(grib_template_derive::FieldOffset)]
struct TemplateB<T> {
    prefix: T,
    #[field_offset(Param)]
    param: Param,
}

#[derive(grib_template_derive::FixedSize)]
struct Packed<T, const N: usize> {
    prefix: T,
    #[grib_template(num_octets = 3)]
    packed: u32,
    bytes: [u8; N],
}

#[test]
fn encoded_sizes_ignore_memory_padding() {
    assert_eq!(Param::SIZE, 3);
    assert_eq!(Packed::<Param, 2>::SIZE, 8);
    assert_eq!(<TemplateA as FieldOffset<Param>>::OFFSET, 4);
    assert_eq!(<TemplateB<u16> as FieldOffset<Param>>::OFFSET, 2);
}

#[test]
fn reads_without_deserializing_missing_tail() -> Result<(), Box<dyn std::error::Error>> {
    let buf = [0, 1, 2, 3, 4, 5, 6];
    assert!(TemplateA::try_from_slice(&buf, &mut 0).is_err());
    assert_eq!(
        TemplateA::read_fixed_field(&buf)?,
        Param {
            category: 4,
            number: 0x0506
        }
    );
    assert_eq!(
        TemplateB::<u16>::read_fixed_field(&buf[2..])?,
        Param {
            category: 4,
            number: 0x0506
        }
    );
    Ok(())
}

#[test]
fn short_selected_field_returns_error() {
    for len in 0..7 {
        assert!(TemplateA::read_fixed_field(&[0; 7][..len]).is_err());
    }
}
