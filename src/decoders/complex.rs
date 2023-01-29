use num::ToPrimitive;
use std::convert::TryInto;
use std::iter;

use crate::decoders::common::*;
use crate::decoders::simple::*;
use crate::error::*;
use crate::utils::{read_as, GribInt, NBitwiseIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComplexPackingDecodeError {
    NotSupported,
    LengthMismatch,
}

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIteratorWrapper<impl Iterator<Item = i32> + '_>, GribError> {
    let sect5_data = &target.sect5_payload;
    let ref_val = read_as!(f32, sect5_data, 6);
    let exp = read_as!(u16, sect5_data, 10).as_grib_int();
    let dig = read_as!(u16, sect5_data, 12).as_grib_int();
    let nbit = read_as!(u8, sect5_data, 14);
    let group_splitting_method_used = read_as!(u8, sect5_data, 16);
    let missing_value_management_used = read_as!(u8, sect5_data, 17);
    let ngroup = read_as!(u32, sect5_data, 26);
    let group_width_ref = read_as!(u8, sect5_data, 30);
    let group_width_nbit = read_as!(u8, sect5_data, 31);
    let group_len_ref = read_as!(u32, sect5_data, 32);
    let group_len_inc = read_as!(u8, sect5_data, 36);
    let group_len_last = read_as!(u32, sect5_data, 37);
    let group_len_nbit = read_as!(u8, sect5_data, 41);
    let spdiff_level = read_as!(u8, sect5_data, 42);
    let spdiff_param_octet = read_as!(u8, sect5_data, 43);

    if nbit == 0 {
        let decoder = SimplePackingDecodeIteratorWrapper::FixedValue(FixedValueIterator::new(
            ref_val,
            target.num_points_encoded,
        ));
        return Ok(decoder);
    };

    if group_splitting_method_used != 1 || missing_value_management_used != 0 {
        return Err(GribError::DecodeError(
            DecodeError::ComplexPackingDecodeError(ComplexPackingDecodeError::NotSupported),
        ));
    }

    let sect7_data = &target.sect7_payload;
    let sect7_params = section7::SpatialDifferencingExtraDescriptors::new(
        sect7_data,
        spdiff_level,
        spdiff_param_octet,
    )?;

    fn get_octet_length(nbit: u8, ngroup: u32) -> usize {
        let total_bit: u32 = ngroup * u32::from(nbit);
        let total_octet = (total_bit + 0b111) >> 3;
        total_octet as usize
    }

    let params_end_octet = sect7_params.len();
    let group_refs_end_octet = params_end_octet + get_octet_length(nbit, ngroup);
    let group_widths_end_octet = group_refs_end_octet + get_octet_length(group_width_nbit, ngroup);
    let group_lens_end_octet = group_widths_end_octet + get_octet_length(group_len_nbit, ngroup);

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
        .map(move |v| u32::from(group_width_ref) + v);

    let group_lens_iter = NBitwiseIterator::new(
        &sect7_data[group_widths_end_octet..group_lens_end_octet],
        usize::from(group_len_nbit),
    );
    let group_lens_iter = group_lens_iter
        .take((ngroup - 1) as usize)
        .map(move |v| group_len_ref + u32::from(group_len_inc) * v)
        .chain(iter::once(group_len_last));

    let unpacked_data = ComplexPackingValueDecodeIterator::new(
        group_refs_iter,
        group_widths_iter,
        group_lens_iter,
        sect7_params.minimum(),
        sect7_data[group_lens_end_octet..].to_vec(),
    );

    let spdiff_packed_iter = unpacked_data.flatten();
    let first_values = sect7_params.first_values().collect::<Vec<_>>();
    let num_first_values = first_values.len();
    let spdiff_packed_iter = first_values
        .into_iter()
        .chain(spdiff_packed_iter.skip(num_first_values));

    let spdiff_unpacked = SpatialDiff2ndOrderDecodeIterator::new(spdiff_packed_iter);
    let decoder = SimplePackingDecodeIterator::new(spdiff_unpacked, ref_val, exp, dig);
    let decoder = SimplePackingDecodeIteratorWrapper::SimplePacking(decoder);
    Ok(decoder)
}

#[derive(Clone)]
struct ComplexPackingValueDecodeIterator<I, J, K> {
    ref_iter: I,
    width_iter: J,
    length_iter: K,
    z_min: i32,
    data: Vec<u8>,
    pos: usize,
    start_offset_bits: usize,
}

impl<I, J, K> ComplexPackingValueDecodeIterator<I, J, K> {
    pub(crate) fn new(
        ref_iter: I,
        width_iter: J,
        length_iter: K,
        z_min: i32,
        data: Vec<u8>,
    ) -> Self {
        Self {
            ref_iter,
            width_iter,
            length_iter,
            z_min,
            data,
            pos: 0,
            start_offset_bits: 0,
        }
    }
}

impl<I: Iterator<Item = N>, J: Iterator<Item = O>, K: Iterator<Item = P>, N, O, P> Iterator
    for ComplexPackingValueDecodeIterator<I, J, K>
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
            (Some(_ref), Some(width), Some(length)) if width.to_usize().unwrap() == 0 => {
                // The specification states as follows: "For groups with a constant value,
                // associated field width is 0, and no incremental data are physically present."
                let _ref = _ref.to_i32().unwrap();
                let length = length.to_usize().unwrap();
                Some(vec![_ref + self.z_min; length])
            }
            (Some(_ref), Some(width), Some(length)) => {
                let (_ref, width, length) = (
                    _ref.to_i32().unwrap(),
                    width.to_usize().unwrap(),
                    length.to_usize().unwrap(),
                );
                let bits = self.start_offset_bits + width * length;
                let (pos_end, offset_bits) = (self.pos + bits / 8, bits % 8);
                let offset_byte = usize::from(offset_bits > 0);
                let group_values =
                    NBitwiseIterator::new(&self.data[self.pos..pos_end + offset_byte], width)
                        .with_offset(self.start_offset_bits)
                        .take(length)
                        .map(|v| v.as_grib_int() + _ref + self.z_min)
                        .collect::<Vec<i32>>();
                self.pos = pos_end;
                self.start_offset_bits = offset_bits;
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

mod section7;
