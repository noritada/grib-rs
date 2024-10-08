use std::iter;

use num::ToPrimitive;

use self::{
    diff::{
        FirstOrderSpatialDifferencingDecodeIterator, SecondOrderSpatialDifferencingDecodeIterator,
        SpatialDifferencingDecodeIterator,
    },
    missing::DecodedValue::{self, Missing1, Missing2, Normal},
};
use crate::{
    codetables::grib2::Table5_6,
    decoder::{
        param::{ComplexPackingParam, SimplePackingParam},
        simple::*,
        stream::{BitStream, NBitwiseIterator},
        DecodeError, Grib2SubmessageDecoder,
    },
    error::*,
    helpers::{read_as, GribInt},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComplexPackingDecodeError {
    NotSupported,
    LengthMismatch,
}

pub(crate) fn decode_7_2(
    target: &Grib2SubmessageDecoder,
) -> Result<
    SimplePackingDecodeIteratorWrapper<impl Iterator<Item = DecodedValue<i32>> + '_>,
    GribError,
> {
    let sect5_data = &target.sect5_payload;
    let simple_param = SimplePackingParam::from_buf(&sect5_data[6..16])?;
    let complex_param = ComplexPackingParam::from_buf(&sect5_data[16..42]);

    if complex_param.group_splitting_method_used != 1
        || complex_param.missing_value_management_used > 2
    {
        return Err(GribError::DecodeError(
            DecodeError::ComplexPackingDecodeError(ComplexPackingDecodeError::NotSupported),
        ));
    }

    let sect7_data = &target.sect7_payload;

    let unpacked_data = decode_complex_packing(complex_param, sect7_data, 0, simple_param.nbit, 0);
    let decoder = SimplePackingDecodeIterator::new(unpacked_data, &simple_param);
    let decoder = SimplePackingDecodeIteratorWrapper::SimplePacking(decoder);
    Ok(decoder)
}

pub(crate) fn decode_7_3(
    target: &Grib2SubmessageDecoder,
) -> Result<
    SimplePackingDecodeIteratorWrapper<impl Iterator<Item = DecodedValue<i32>> + '_>,
    GribError,
> {
    let sect5_data = &target.sect5_payload;
    let simple_param = SimplePackingParam::from_buf(&sect5_data[6..16])?;
    let complex_param = ComplexPackingParam::from_buf(&sect5_data[16..42]);
    let spdiff_order = read_as!(u8, sect5_data, 42);
    let spdiff_order = Table5_6::try_from(spdiff_order).map_err(|e| {
        let number = e.number;
        GribError::NotSupported(format!("Code Table 5.6 value '{number}' is not supported"))
    })?;
    let spdiff_param_octet = read_as!(u8, sect5_data, 43);

    if complex_param.group_splitting_method_used != 1
        || complex_param.missing_value_management_used > 2
        || matches!(spdiff_order, Table5_6::Missing)
    {
        return Err(GribError::DecodeError(
            DecodeError::ComplexPackingDecodeError(ComplexPackingDecodeError::NotSupported),
        ));
    }

    let sect7_data = &target.sect7_payload;
    let sect7_params = diff::SpatialDifferencingExtraDescriptors::new(
        sect7_data,
        u8::from(spdiff_order.clone()),
        spdiff_param_octet,
    )?;

    let unpacked_data = decode_complex_packing(
        complex_param,
        sect7_data,
        sect7_params.len(),
        simple_param.nbit,
        sect7_params.minimum(),
    );
    let first_values = sect7_params.first_values();
    let first_values = first_values.collect::<Vec<_>>().into_iter();
    let spdiff_unpacked = match spdiff_order {
        Table5_6::FirstOrderSpatialDifferencing => SpatialDifferencingDecodeIterator::FirstOrder(
            FirstOrderSpatialDifferencingDecodeIterator::new(unpacked_data, first_values),
        ),
        Table5_6::SecondOrderSpatialDifferencing => SpatialDifferencingDecodeIterator::SecondOrder(
            SecondOrderSpatialDifferencingDecodeIterator::new(unpacked_data, first_values),
        ),
        Table5_6::Missing => unreachable!(),
    };
    let decoder = SimplePackingDecodeIterator::new(spdiff_unpacked, &simple_param);
    let decoder = SimplePackingDecodeIteratorWrapper::SimplePacking(decoder);
    Ok(decoder)
}

fn decode_complex_packing(
    complex_param: ComplexPackingParam,
    sect7_data: &[u8],
    sect7_offset: usize,
    nbit: u8,
    z_min: i32,
) -> impl Iterator<Item = DecodedValue<i32>> + '_ {
    fn get_octet_length(nbit: u8, ngroup: u32) -> usize {
        let total_bit: u32 = ngroup * u32::from(nbit);
        let total_octet = (total_bit + 0b111) >> 3;
        total_octet as usize
    }

    let params_end_octet = sect7_offset;
    let group_refs_end_octet = params_end_octet + get_octet_length(nbit, complex_param.ngroup);
    let group_widths_end_octet = group_refs_end_octet
        + get_octet_length(complex_param.group_width_nbit, complex_param.ngroup);
    let group_lens_end_octet = group_widths_end_octet
        + get_octet_length(complex_param.group_len_nbit, complex_param.ngroup);

    let group_refs_iter = BitStream::new(
        &sect7_data[params_end_octet..group_refs_end_octet],
        usize::from(nbit),
        complex_param.ngroup as usize,
    );
    let group_refs_iter = group_refs_iter.take(complex_param.ngroup as usize);

    let group_widths_iter = BitStream::new(
        &sect7_data[group_refs_end_octet..group_widths_end_octet],
        usize::from(complex_param.group_width_nbit),
        complex_param.ngroup as usize,
    );
    let group_widths_iter = group_widths_iter
        .take(complex_param.ngroup as usize)
        .map(move |v| u32::from(complex_param.group_width_ref) + v);

    let group_lens_iter = BitStream::new(
        &sect7_data[group_widths_end_octet..group_lens_end_octet],
        usize::from(complex_param.group_len_nbit),
        (complex_param.ngroup - 1) as usize,
    );
    let group_lens_iter = group_lens_iter
        .take((complex_param.ngroup - 1) as usize)
        .map(move |v| complex_param.group_len_ref + u32::from(complex_param.group_len_inc) * v)
        .chain(iter::once(complex_param.group_len_last));

    ComplexPackingValueDecodeIterator::new(
        group_refs_iter,
        group_widths_iter,
        group_lens_iter,
        complex_param.missing_value_management_used,
        nbit,
        z_min,
        sect7_data[group_lens_end_octet..].to_vec(),
    )
    .flatten()
}

#[derive(Clone)]
struct ComplexPackingValueDecodeIterator<I, J, K> {
    ref_iter: I,
    width_iter: J,
    length_iter: K,
    missing_value_management: u8,
    nbit: u8,
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
        missing_value_management: u8,
        nbit: u8,
        z_min: i32,
        data: Vec<u8>,
    ) -> Self {
        Self {
            ref_iter,
            width_iter,
            length_iter,
            missing_value_management,
            nbit,
            z_min,
            data,
            pos: 0,
            start_offset_bits: 0,
        }
    }
}

impl<I, J, K, N, O, P> Iterator for ComplexPackingValueDecodeIterator<I, J, K>
where
    I: Iterator<Item = N>,
    J: Iterator<Item = O>,
    K: Iterator<Item = P>,
    N: ToPrimitive,
    O: ToPrimitive,
    P: ToPrimitive,
{
    type Item = Vec<DecodedValue<i32>>;

    fn next(&mut self) -> Option<Self::Item> {
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
                let missing1 = (1 << self.nbit) - 1;
                let missing2 = missing1 - 1;

                if self.missing_value_management > 0 && _ref == missing1 {
                    Some(vec![Missing1; length])
                } else if self.missing_value_management == 2 && _ref == missing2 {
                    Some(vec![Missing2; length])
                } else {
                    Some(vec![Normal(_ref + self.z_min); length])
                }
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
                let missing1 = (1 << width) - 1;
                let missing2 = missing1 - 1;
                let group_values =
                    NBitwiseIterator::new(&self.data[self.pos..pos_end + offset_byte], width)
                        .with_offset(self.start_offset_bits)
                        .take(length)
                        .map(|v| {
                            if self.missing_value_management > 0 && v == missing1 {
                                Missing1
                            } else if self.missing_value_management == 2 && v == missing2 {
                                Missing2
                            } else {
                                Normal(v.as_grib_int() + _ref + self.z_min)
                            }
                        })
                        .collect::<Vec<_>>();
                self.pos = pos_end;
                self.start_offset_bits = offset_bits;
                Some(group_values)
            }
            _ => None,
        }
    }
}

mod diff;
mod missing;
