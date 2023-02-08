use std::convert::TryInto;

use crate::{
    decoders::common::*,
    error::*,
    utils::{read_as, NBitwiseIterator},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunLengthEncodingDecodeError {
    NotSupported,
    InvalidFirstValue,
    LengthMismatch,
    InvalidLevelValue(u16),
}

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<std::vec::IntoIter<f32>, GribError> {
    let sect5_data = &target.sect5_payload;
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

    let decoded_levels = rleunpack(
        &target.sect7_payload,
        nbit,
        maxv,
        Some(target.num_points_encoded),
    )
    .map_err(DecodeError::RunLengthEncodingDecodeError)?;

    let level_to_value = |level: &u16| -> Result<f32, DecodeError> {
        let index: usize = (*level).into();
        level_map
            .get(index)
            .copied()
            .ok_or(DecodeError::RunLengthEncodingDecodeError(
                RunLengthEncodingDecodeError::InvalidLevelValue(*level),
            ))
    };

    let decoded: Result<Vec<_>, _> = (*decoded_levels).iter().map(level_to_value).collect();
    Ok(decoded?.into_iter())
}

// Since maxv is represented as a 16-bit integer, values are 16 bits or less.
fn rleunpack(
    input: &[u8],
    nbit: u8,
    maxv: u16,
    expected_len: Option<usize>,
) -> Result<Box<[u16]>, RunLengthEncodingDecodeError> {
    let mut out_buf = match expected_len {
        Some(sz) => Vec::with_capacity(sz),
        None => Vec::new(),
    };

    let rlbase = maxv + 1;
    let lngu: usize = ((1u16 << nbit) - rlbase).into();
    let mut cached = None;
    let mut exp: usize = 1;
    let iter = NBitwiseIterator::new(input, usize::from(nbit));

    for value in iter {
        let value = value as u16;
        if rlbase > value {
            out_buf.push(value);
            cached = Some(value);
            exp = 1;
        } else {
            let prev = cached.ok_or(RunLengthEncodingDecodeError::InvalidFirstValue)?;
            let length: usize = ((value - rlbase) as usize) * exp;
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

#[cfg(test)]
mod tests {
    use super::*;

    // This test data is based on 4-bit numbers of sample data
    // provided in JMBSC's document available in the following URL:
    // http://www.jmbsc.or.jp/jp/online/joho-sample/Run-Length_Encoding.pdf
    #[test]
    fn decode_data_with_run_length_encoding() {
        let input: Vec<u8> = vec![3, 9, 12, 6, 4, 15, 2, 1, 0, 13, 12, 2, 3];
        let output: Vec<u16> = vec![
            3, 9, 9, 6, 4, 4, 4, 4, 4, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3,
        ];
        let input: Vec<u8> = input.iter().map(|n| n + 240).collect();
        let output: Vec<u16> = output.iter().map(|n| n + 240).collect();

        assert_eq!(
            rleunpack(&input, 8, 250, Some(21)),
            Ok(output.into_boxed_slice())
        );
    }

    #[test]
    fn decode_data_with_run_length_encoding_with_multibyte_length() {
        let input: Vec<u8> = vec![0x00, 0x14, 0x1c];
        let output: Vec<u16> = vec![0; 6065];

        assert_eq!(rleunpack(&input, 8, 3, None), Ok(output.into_boxed_slice()));
    }
}
