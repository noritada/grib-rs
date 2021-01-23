use num::ToPrimitive;
use std::cell::RefMut;
use std::convert::TryInto;

use crate::context::{GribError, SectionBody, SectionInfo};
use crate::reader::Grib2Read;
use crate::utils::{GribInt, NBitwiseIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecodeError {
    TemplateNumberUnsupported,
    BitMapIndicatorUnsupported,
    SimplePackingDecodeError(SimplePackingDecodeError),
    ComplexPackingDecodeError(ComplexPackingDecodeError),
    RunLengthEncodingDecodeError(RunLengthEncodingDecodeError),
}

impl From<SimplePackingDecodeError> for DecodeError {
    fn from(e: SimplePackingDecodeError) -> Self {
        Self::SimplePackingDecodeError(e)
    }
}

impl From<ComplexPackingDecodeError> for DecodeError {
    fn from(e: ComplexPackingDecodeError) -> Self {
        Self::ComplexPackingDecodeError(e)
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
pub enum ComplexPackingDecodeError {
    NotSupported,
    LengthMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunLengthEncodingDecodeError {
    NotSupported,
    InvalidFirstValue,
    LengthMismatch,
    InvalidLevelValue(u16),
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
        3 => ComplexPackingDecoder::decode(sect5, sect6, sect7, reader)?,
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

        let level_to_value = |level: &u16| -> Result<f32, DecodeError> {
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
    let lngu: usize = (2u16.pow(nbit.into()) - rlbase).into();
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

struct ComplexPackingDecoder {}

impl<R: Grib2Read> Grib2DataDecode<R> for ComplexPackingDecoder {
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
        let ngroup = read_as!(u32, sect5_data, 26);
        let group_width_ref = read_as!(u8, sect5_data, 30);
        let group_width_nbit = read_as!(u8, sect5_data, 31);
        let group_len_ref = read_as!(u32, sect5_data, 32);
        let group_len_inc = read_as!(u8, sect5_data, 36);
        let group_len_last = read_as!(u32, sect5_data, 37);
        let group_len_nbit = read_as!(u8, sect5_data, 41);
        let spdiff_level = read_as!(u8, sect5_data, 42);
        let spdiff_param_octet = read_as!(u8, sect5_data, 43);

        let params_end_octet = 6;

        let group_refs_nbit: u32 = ngroup * u32::from(nbit);
        let group_refs_noctet: f32 = group_refs_nbit as f32 / 8_f32;
        let group_refs_noctet = group_refs_noctet.ceil() as usize;
        let group_refs_end_octet = params_end_octet + group_refs_noctet;

        let group_widths_nbit: u32 = ngroup * u32::from(group_width_nbit);
        let group_widths_noctet: f32 = group_widths_nbit as f32 / 8_f32;
        let group_widths_noctet = group_widths_noctet.ceil() as usize;
        let group_widths_end_octet = group_refs_end_octet + group_widths_noctet;

        let group_lens_nbit: u32 = ngroup * u32::from(group_len_nbit);
        let group_lens_noctet: f32 = group_lens_nbit as f32 / 8_f32;
        let group_lens_noctet = group_lens_noctet.ceil() as usize;
        let group_lens_end_octet = group_widths_end_octet + group_lens_noctet;

        let sect7_data = reader.read_sect_body_bytes(sect7)?;

        let z1 = read_as!(u16, sect7_data, 0).into_grib_int();
        let z2 = read_as!(u16, sect7_data, 2).into_grib_int();
        let z_min = read_as!(u16, sect7_data, 4).into_grib_int();

        let group_ref_iter = NBitwiseIterator::new(
            &sect7_data[params_end_octet..group_refs_end_octet],
            usize::from(nbit),
        );
        let group_refs_iter = group_ref_iter.take(ngroup as usize);

        let group_width_iter = NBitwiseIterator::new(
            &sect7_data[group_refs_end_octet..group_widths_end_octet],
            usize::from(group_width_nbit),
        );
        let group_widths_iter = group_width_iter
            .take(ngroup as usize)
            .map(|v| u32::from(group_width_ref) + v);

        let group_len_iter = NBitwiseIterator::new(
            &sect7_data[group_widths_end_octet..group_lens_end_octet],
            usize::from(group_len_nbit),
        );
        let mut group_lens = group_len_iter
            .take((ngroup - 1) as usize)
            .map(|v| u32::from(group_len_ref) + u32::from(group_len_inc) * v)
            .collect::<Vec<_>>();
        group_lens.push(group_len_last);

        let unpacked_data = ComplexPackingValueDecodeIterator::new(
            group_refs_iter,
            group_widths_iter,
            group_lens.into_iter(),
            z_min,
            &sect7_data[group_lens_end_octet..],
        );
        let unpacked_data = unpacked_data.collect::<Vec<_>>();

        if spdiff_level != 2 {
            return Err(GribError::DecodeError(
                DecodeError::ComplexPackingDecodeError(ComplexPackingDecodeError::NotSupported),
            ));
        }

        if spdiff_param_octet != 2 {
            return Err(GribError::DecodeError(
                DecodeError::ComplexPackingDecodeError(ComplexPackingDecodeError::NotSupported),
            ));
        }

        let spdiff_packed_iter = unpacked_data.into_iter().flatten();
        assert_eq!(
            spdiff_packed_iter.clone().take(2).collect::<Vec<_>>(),
            [i32::from(z1), i32::from(z2)]
        );

        let spdiff_unpacked = SpatialDiff2ndOrderDecodeIterator::new(spdiff_packed_iter);
        let decoded = SimplePackingDecodeIterator::new(spdiff_unpacked, ref_val, exp, dig)
            .collect::<Vec<_>>();
        if decoded.len() != sect5_body.num_points as usize {
            return Err(GribError::DecodeError(
                DecodeError::SimplePackingDecodeError(SimplePackingDecodeError::LengthMismatch),
            ));
        }
        Ok(decoded.into_boxed_slice())
    }
}

struct SimplePackingDecodeIterator<I> {
    iter: I,
    ref_val: f32,
    exp: i32,
    dig: i32,
}

impl<I> SimplePackingDecodeIterator<I> {
    fn new(iter: I, ref_val: f32, exp: i16, dig: i16) -> Self {
        Self {
            iter: iter,
            ref_val: ref_val,
            exp: exp.into(),
            dig: dig.into(),
        }
    }
}

impl<I: Iterator<Item = N>, N: ToPrimitive> Iterator for SimplePackingDecodeIterator<I> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        match self.iter.next() {
            None => None,
            Some(encoded) => {
                let encoded = encoded.to_f32().unwrap();
                let diff = (encoded * 2_f32.powi(self.exp)) as f32;
                let dig_factor = 10_f32.powi(-self.dig);
                let value: f32 = (self.ref_val + diff) * dig_factor;
                Some(value)
            }
        }
    }
}

struct ComplexPackingValueDecodeIterator<'a, I, J, K> {
    ref_iter: I,
    width_iter: J,
    length_iter: K,
    z_min: i32,
    data: &'a [u8],
    pos: usize,
    start_offset_bits: usize,
}

impl<'a, I, J, K> ComplexPackingValueDecodeIterator<'a, I, J, K> {
    fn new(ref_iter: I, width_iter: J, length_iter: K, z_min: i16, data: &'a [u8]) -> Self {
        Self {
            ref_iter: ref_iter,
            width_iter: width_iter,
            length_iter: length_iter,
            z_min: i32::from(z_min),
            data: data,
            pos: 0,
            start_offset_bits: 0,
        }
    }
}

impl<'a, I: Iterator<Item = N>, J: Iterator<Item = O>, K: Iterator<Item = P>, N, O, P> Iterator
    for ComplexPackingValueDecodeIterator<'a, I, J, K>
where
    N: ToPrimitive,
    O: ToPrimitive,
    P: ToPrimitive,
{
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Vec<i32>> {
        match (
            self.ref_iter.next(),
            self.width_iter.next(),
            self.length_iter.next(),
        ) {
            (None, _, _) => None,
            (_, None, _) => None,
            (_, _, None) => None,
            (Some(_ref), Some(width), Some(length)) => {
                let (_ref, width, length) = (
                    _ref.to_i32().unwrap(),
                    width.to_usize().unwrap(),
                    length.to_usize().unwrap(),
                );
                let bits = width * length;
                let (pos_end, offset_bit) = (self.pos + bits / 8, bits % 8);
                let offset_byte = if offset_bit > 0 { 1 } else { 0 };
                let group_values =
                    NBitwiseIterator::new(&self.data[self.pos..pos_end + offset_byte], width)
                        .with_offset(self.start_offset_bits)
                        .take(length)
                        .map(|v| v.into_grib_int() + _ref + self.z_min)
                        .collect::<Vec<i32>>();
                self.pos = pos_end;
                self.start_offset_bits = offset_byte;
                Some(group_values)
            }
        }
    }
}

struct SpatialDiff2ndOrderDecodeIterator<I> {
    iter: I,
    count: u32,
    prev1: i32,
    prev2: i32,
}

impl<I> SpatialDiff2ndOrderDecodeIterator<I> {
    fn new(iter: I) -> Self {
        Self {
            iter: iter,
            count: 0,
            prev1: 0,
            prev2: 0,
        }
    }
}

impl<I: Iterator<Item = i32>> Iterator for SpatialDiff2ndOrderDecodeIterator<I> {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        let count = self.count;
        self.count += 1;
        match (count, self.iter.next()) {
            (_, None) => None,
            (0, Some(v)) => {
                self.prev2 = v;
                Some(v)
            }
            (1, Some(v)) => {
                self.prev1 = v;
                Some(v)
            }
            (_, Some(v)) => {
                let v = v + 2 * self.prev1 - self.prev2;
                self.prev2 = self.prev1;
                self.prev1 = v;
                Some(v)
            }
        }
    }
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

    #[test]
    fn decode_simple_packing() {
        let ref_val_bytes = vec![0x35, 0x3e, 0x6b, 0xf6];
        let exp = 0x801a;
        let dig = 0x0000;
        let input: Vec<u8> = vec![0x00, 0x06, 0x00, 0x0d];
        let expected: Vec<f32> = vec![7.98783162e-07, 9.03091291e-07];

        let ref_val = f32::from_be_bytes(ref_val_bytes[..].try_into().unwrap());
        let iter = NBitwiseIterator::new(&input, 16);
        let actual = SimplePackingDecodeIterator::new(
            iter,
            ref_val,
            exp.into_grib_int(),
            dig.into_grib_int(),
        )
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
