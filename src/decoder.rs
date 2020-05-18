use std::cell::RefMut;
use std::convert::TryInto;

use crate::data::{GribError, SectionBody, SectionInfo};
use crate::reader::Grib2Read;
use crate::utils::GribInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecodeError {
    TemplateNumberUnsupported,
    BitMapIndicatorUnsupported,
    SimplePackingDecodeError(SimplePackingDecodeError),
    RunLengthEncodingDecodeError(RunLengthEncodingDecodeError),
}

impl From<SimplePackingDecodeError> for DecodeError {
    fn from(e: SimplePackingDecodeError) -> Self {
        Self::SimplePackingDecodeError(e)
    }
}

impl From<RunLengthEncodingDecodeError> for DecodeError {
    fn from(e: RunLengthEncodingDecodeError) -> Self {
        Self::RunLengthEncodingDecodeError(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimplePackingDecodeError {
    NotSupported,
    OriginalFieldValueTypeNotSupported,
    LengthMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunLengthEncodingDecodeError {
    NotSupported,
    InvalidFirstValue,
    LengthMismatch,
    InvalidLevelValue(u8),
}

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

pub fn dispatch<R: Grib2Read>(
    sect5: &SectionInfo,
    sect6: &SectionInfo,
    sect7: &SectionInfo,
    reader: RefMut<R>,
) -> Result<Box<[f32]>, GribError> {
    let sect5_body = match &sect5.body {
        Some(SectionBody::Section5(body)) => body,
        _ => return Err(GribError::InternalDataError),
    };

    let decoded = match sect5_body.repr_tmpl_num {
        0 => SimplePackingDecoder::decode(sect5, sect6, sect7, reader)?,
        200 => RunLengthEncodingDecoder::decode(sect5, sect6, sect7, reader)?,
        _ => {
            return Err(GribError::DecodeError(
                DecodeError::TemplateNumberUnsupported,
            ))
        }
    };
    Ok(decoded)
}

trait Grib2DataDecode<R> {
    fn decode(
        sect5: &SectionInfo,
        sect6: &SectionInfo,
        sect7: &SectionInfo,
        reader: RefMut<R>,
    ) -> Result<Box<[f32]>, GribError>;
}

struct RunLengthEncodingDecoder {}

impl<R: Grib2Read> Grib2DataDecode<R> for RunLengthEncodingDecoder {
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
        let nbit = read_as!(u8, sect5_data, 6);
        let maxv = read_as!(u16, sect5_data, 7);
        let max_level = read_as!(u16, sect5_data, 9);
        let num_digits = read_as!(u8, sect5_data, 11);

        let mut level_map = Vec::with_capacity(max_level.into());
        level_map.push(f32::NAN);
        let mut pos = 12;

        for _ in 0..max_level {
            let val: f32 = read_as!(u16, sect5_data, pos).into();
            let num_digits: i32 = num_digits.into();
            let factor = 10_f32.powi(-num_digits);
            let val = val * factor;
            level_map.push(val);
            pos += std::mem::size_of::<u16>();
        }

        let sect7_data = reader.read_sect_body_bytes(sect7)?;

        let decoded_levels = rleunpack(
            &sect7_data,
            nbit,
            maxv,
            Some(sect5_body.num_points as usize),
        )
        .map_err(|e| DecodeError::RunLengthEncodingDecodeError(e))?;

        let level_to_value = |level: &u8| -> Result<f32, DecodeError> {
            let index: usize = (*level).into();
            level_map
                .get(index)
                .map(|l| *l)
                .ok_or(DecodeError::RunLengthEncodingDecodeError(
                    RunLengthEncodingDecodeError::InvalidLevelValue(*level),
                ))
        };

        let decoded: Result<Vec<_>, _> = (*decoded_levels).iter().map(level_to_value).collect();
        let decoded = decoded?.into_boxed_slice();
        Ok(decoded)
    }
}

fn rleunpack(
    input: &[u8],
    nbit: u8,
    maxv: u16,
    expected_len: Option<usize>,
) -> Result<Box<[u8]>, RunLengthEncodingDecodeError> {
    if nbit != 8 {
        return Err(RunLengthEncodingDecodeError::NotSupported);
    }

    let mut out_buf = match expected_len {
        Some(sz) => Vec::with_capacity(sz),
        None => Vec::new(),
    };

    let rlbase = maxv + 1;
    let lngu: usize = (2u16.pow(nbit.into()) - rlbase).into();
    let mut cached = None;
    let mut exp: usize = 1;

    for value in input.iter() {
        let value = *value;

        if rlbase > value.into() {
            out_buf.push(value);
            cached = Some(value);
            exp = 1;
        } else {
            let prev = cached.ok_or(RunLengthEncodingDecodeError::InvalidFirstValue)?;
            let length: usize = ((value as u16 - rlbase) as usize) * exp;
            out_buf.append(&mut vec![prev; length]);
            exp *= lngu;
        }
    }

    if let Some(len) = expected_len {
        if len != out_buf.len() {
            return Err(RunLengthEncodingDecodeError::LengthMismatch);
        }
    }

    Ok(out_buf.into_boxed_slice())
}

struct SimplePackingDecoder {}

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
        let exp = read_as!(u16, sect5_data, 10).into_grib_int();
        let dig = read_as!(u16, sect5_data, 12).into_grib_int();
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

        let decoded = unpack_simple_packing(
            &sect7_data,
            nbit,
            ref_val,
            exp,
            dig,
            Some(sect5_body.num_points as usize),
        )
        .map_err(|e| DecodeError::SimplePackingDecodeError(e))?;
        Ok(decoded)
    }
}

fn unpack_simple_packing(
    input: &[u8],
    nbit: u8,
    ref_val: f32,
    exp: i16,
    dig: i16,
    expected_len: Option<usize>,
) -> Result<Box<[f32]>, SimplePackingDecodeError> {
    if nbit != 16 {
        return Err(SimplePackingDecodeError::NotSupported);
    }

    let mut out_buf = match expected_len {
        Some(sz) => Vec::with_capacity(sz),
        None => Vec::new(),
    };

    let dig: i32 = dig.into();
    let mut pos = 0;

    while pos < input.len() {
        let encoded = read_as!(u16, input, pos) as f32;
        pos += std::mem::size_of::<u16>();

        let diff = (encoded * 2_f32.powi(exp.into())) as f32;
        let dig_factor = 10_f32.powi(-dig);
        let value: f32 = (ref_val + diff) * dig_factor;
        out_buf.push(value);
    }

    if let Some(len) = expected_len {
        if len != out_buf.len() {
            return Err(SimplePackingDecodeError::LengthMismatch);
        }
    }

    Ok(out_buf.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    // This test data is based on 4-bit numbers of sample data
    // provided in JMBSC's document available in the following URL:
    // http://www.jmbsc.or.jp/jp/online/joho-sample/Run-Length_Encoding.pdf
    #[test]
    fn rleunpack_u8() {
        let input: Vec<u8> = vec![3, 9, 12, 6, 4, 15, 2, 1, 0, 13, 12, 2, 3];
        let output: Vec<u8> = vec![
            3, 9, 9, 6, 4, 4, 4, 4, 4, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3,
        ];
        let input: Vec<u8> = input.iter().map(|n| n + 240).collect();
        let output: Vec<u8> = output.iter().map(|n| n + 240).collect();

        assert_eq!(
            rleunpack(&input, 8, 250, Some(21)),
            Ok(output.into_boxed_slice())
        );
    }

    #[test]
    fn rleunpack_u8_long_length() {
        let input: Vec<u8> = vec![0x00, 0x14, 0x1c];
        let output: Vec<u8> = vec![0; 6065];

        assert_eq!(rleunpack(&input, 8, 3, None), Ok(output.into_boxed_slice()));
    }

    #[test]
    fn simple_packing_u8() {
        let ref_val_bytes = vec![0x35, 0x3e, 0x6b, 0xf6];
        let exp = 0x801a;
        let dig = 0x0000;
        let input: Vec<u8> = vec![0x00, 0x06, 0x00, 0x0d];
        let expected: Vec<f32> = vec![7.98783162e-07, 9.03091291e-07];

        let ref_val = f32::from_be_bytes(ref_val_bytes[..].try_into().unwrap());
        let actual = unpack_simple_packing(
            &input,
            16,
            ref_val,
            exp.into_grib_int(),
            dig.into_grib_int(),
            Some(2),
        )
        .unwrap();

        assert_eq!(actual.len(), expected.len());
        let mut i = 0;
        while i < actual.len() {
            assert!(actual[i] < expected[i] + 0.00000001);
            assert!(actual[i] > expected[i] - 0.00000001);
            i += 1;
        }
    }
}
