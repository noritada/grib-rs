use std::cell::RefMut;
use std::convert::TryInto;

use crate::data::{GribError, SectionBody, SectionInfo};
use crate::reader::Grib2Read;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecodeError {
    TemplateNumberUnsupported,
    BitMapIndicatorUnsupported,
    RunLengthEncodingDecodeError(RunLengthEncodingDecodeError),
}

impl From<RunLengthEncodingDecodeError> for DecodeError {
    fn from(e: RunLengthEncodingDecodeError) -> Self {
        Self::RunLengthEncodingDecodeError(e)
    }
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
) -> Result<Box<[Option<f32>]>, GribError> {
    let sect5_body = match &sect5.body {
        Some(SectionBody::Section5(body)) => body,
        _ => return Err(GribError::InternalDataError),
    };

    let decoded = match sect5_body.repr_tmpl_num {
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
    ) -> Result<Box<[Option<f32>]>, GribError>;
}

struct RunLengthEncodingDecoder {}

impl<R: Grib2Read> Grib2DataDecode<R> for RunLengthEncodingDecoder {
    fn decode(
        sect5: &SectionInfo,
        sect6: &SectionInfo,
        sect7: &SectionInfo,
        mut reader: RefMut<R>,
    ) -> Result<Box<[Option<f32>]>, GribError> {
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
        level_map.push(None);
        let mut pos = 12;

        for _ in 0..max_level {
            let val: f32 = read_as!(u16, sect5_data, pos).into();
            let num_digits: f32 = num_digits.into();
            let factor = 10_f32.powf(-num_digits);
            let val = val * factor;
            level_map.push(Some(val));
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

        let level_to_value = |level: &u8| -> Result<Option<f32>, DecodeError> {
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
    let lngu = (2u16.pow(nbit.into()) - rlbase) as usize;
    let mut cached = None;
    let mut exp: usize = 1;

    for value in input.iter() {
        let value = *value;

        if (value as u16) < rlbase {
            out_buf.push(value);
            cached = Some(value);
            exp = 1;
        } else {
            let prev = cached.ok_or(RunLengthEncodingDecodeError::InvalidFirstValue)?;
            let length = ((value as u16 - rlbase) as usize) * exp;
            out_buf.append(&mut vec![prev; length as usize]);
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
}
