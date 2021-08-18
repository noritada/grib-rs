use num::ToPrimitive;
use std::cell::RefMut;
use std::convert::TryInto;
use std::iter;

use crate::context::{SectionBody, SectionInfo};
use crate::decoders::common::*;
use crate::decoders::simple::*;
use crate::error::*;
use crate::reader::Grib2Read;
use crate::utils::{GribInt, NBitwiseIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComplexPackingDecodeError {
    NotSupported,
    LengthMismatch,
}

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

pub(crate) struct ComplexPackingDecoder {}

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

        let sect7_data = reader.read_sect_body_bytes(sect7)?;

        let z1 = read_as!(u16, sect7_data, 0).into_grib_int();
        let z2 = read_as!(u16, sect7_data, 2).into_grib_int();
        let z_min = read_as!(u16, sect7_data, 4).into_grib_int();

        fn get_octet_length(nbit: u8, ngroup: u32) -> usize {
            let total_bit: u32 = ngroup * u32::from(nbit);
            let total_octet: f32 = total_bit as f32 / 8_f32;
            total_octet.ceil() as usize
        }

        let params_end_octet = 6;
        let group_refs_end_octet = params_end_octet + get_octet_length(nbit, ngroup);
        let group_widths_end_octet =
            group_refs_end_octet + get_octet_length(group_width_nbit, ngroup);
        let group_lens_end_octet =
            group_widths_end_octet + get_octet_length(group_len_nbit, ngroup);

        let group_refs_iter = NBitwiseIterator::new(
            &sect7_data[params_end_octet..group_refs_end_octet],
            usize::from(nbit),
        );
        let group_refs_iter = group_refs_iter.take(ngroup as usize);

        let group_widths_iter = NBitwiseIterator::new(
            &sect7_data[group_refs_end_octet..group_widths_end_octet],
            usize::from(group_width_nbit),
        );
        let group_widths_iter = group_widths_iter
            .take(ngroup as usize)
            .map(|v| u32::from(group_width_ref) + v);

        let group_lens_iter = NBitwiseIterator::new(
            &sect7_data[group_widths_end_octet..group_lens_end_octet],
            usize::from(group_len_nbit),
        );
        let group_lens_iter = group_lens_iter
            .take((ngroup - 1) as usize)
            .map(|v| group_len_ref + u32::from(group_len_inc) * v)
            .chain(iter::once(group_len_last));

        let unpacked_data = ComplexPackingValueDecodeIterator::new(
            group_refs_iter,
            group_widths_iter,
            group_lens_iter,
            z_min,
            &sect7_data[group_lens_end_octet..],
        );

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

        let spdiff_packed_iter = unpacked_data.flatten();
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

#[derive(Clone)]
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
    pub(crate) fn new(
        ref_iter: I,
        width_iter: J,
        length_iter: K,
        z_min: i16,
        data: &'a [u8],
    ) -> Self {
        Self {
            ref_iter,
            width_iter,
            length_iter,
            z_min: i32::from(z_min),
            data,
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
            _ => None,
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
            iter,
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
