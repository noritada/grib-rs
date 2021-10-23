use num::ToPrimitive;
use std::cell::RefMut;
use std::convert::TryInto;

use crate::context::{SectionBody, SectionInfo};
use crate::decoders::common::*;
use crate::error::*;
use crate::reader::Grib2Read;
use crate::utils::{GribInt, NBitwiseIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimplePackingDecodeError {
    NotSupported,
    OriginalFieldValueTypeNotSupported,
    LengthMismatch,
}

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

pub(crate) struct SimplePackingDecoder {}

impl<R: Grib2Read> Grib2DataDecode<R> for SimplePackingDecoder {
    fn decode(
        sect5: &SectionInfo,
        sect6: &SectionInfo,
        sect7: &SectionInfo,
        mut reader: RefMut<R>,
    ) -> Result<Box<[f32]>, GribError> {
        let (sect5_body, sect6_body) = match (sect5.body.as_ref(), sect6.body.as_ref()) {
            (Some(SectionBody::Section5(b5)), Some(SectionBody::Section6(b6))) => (b5, b6),
            _ => return Err(GribError::InternalDataError),
        };

        if sect6_body.bitmap_indicator != 255 {
            return Err(GribError::DecodeError(
                DecodeError::BitMapIndicatorUnsupported,
            ));
        }

        let sect5_data = reader.read_sect_body_bytes(sect5)?;
        let ref_val = read_as!(f32, sect5_data, 6);
        let exp = read_as!(u16, sect5_data, 10).as_grib_int();
        let dig = read_as!(u16, sect5_data, 12).as_grib_int();
        let nbit = read_as!(u8, sect5_data, 14);
        let value_type = read_as!(u8, sect5_data, 15);

        if value_type != 0 {
            return Err(GribError::DecodeError(
                DecodeError::SimplePackingDecodeError(
                    SimplePackingDecodeError::OriginalFieldValueTypeNotSupported,
                ),
            ));
        }

        let sect7_data = reader.read_sect_body_bytes(sect7)?;

        let iter = NBitwiseIterator::new(&sect7_data, usize::from(nbit));
        let decoded = SimplePackingDecodeIterator::new(iter, ref_val, exp, dig).collect::<Vec<_>>();
        if decoded.len() != sect5_body.num_points as usize {
            return Err(GribError::DecodeError(
                DecodeError::SimplePackingDecodeError(SimplePackingDecodeError::LengthMismatch),
            ));
        }
        Ok(decoded.into_boxed_slice())
    }
}

pub(crate) struct SimplePackingDecodeIterator<I> {
    iter: I,
    ref_val: f32,
    exp: i32,
    dig: i32,
}

impl<I> SimplePackingDecodeIterator<I> {
    pub(crate) fn new(iter: I, ref_val: f32, exp: i16, dig: i16) -> Self {
        Self {
            iter,
            ref_val,
            exp: exp.into(),
            dig: dig.into(),
        }
    }
}

impl<I: Iterator<Item = N>, N: ToPrimitive> Iterator for SimplePackingDecodeIterator<I> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        match self.iter.next() {
            Some(encoded) => {
                let encoded = encoded.to_f32().unwrap();
                let diff = (encoded * 2_f32.powi(self.exp)) as f32;
                let dig_factor = 10_f32.powi(-self.dig);
                let value: f32 = (self.ref_val + diff) * dig_factor;
                Some(value)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_simple_packing() {
        let ref_val_bytes = vec![0x35, 0x3e, 0x6b, 0xf6];
        let exp = 0x801a;
        let dig = 0x0000;
        let input: Vec<u8> = vec![0x00, 0x06, 0x00, 0x0d];
        let expected: Vec<f32> = vec![7.987_831_6e-7, 9.030_913e-7];

        let ref_val = f32::from_be_bytes(ref_val_bytes[..].try_into().unwrap());
        let iter = NBitwiseIterator::new(&input, 16);
        let actual =
            SimplePackingDecodeIterator::new(iter, ref_val, exp.as_grib_int(), dig.as_grib_int())
                .collect::<Vec<_>>();

        assert_eq!(actual.len(), expected.len());
        let mut i = 0;
        while i < actual.len() {
            assert!(actual[i] < expected[i] + 0.00000001);
            assert!(actual[i] > expected[i] - 0.00000001);
            i += 1;
        }
    }
}
