use std::iter::{self, Take};

use num::ToPrimitive;

pub(crate) use self::diff::SpatialDifferencingDecodeIterator;
use self::{
    diff::{
        FirstOrderSpatialDifferencingDecodeIterator, SecondOrderSpatialDifferencingDecodeIterator,
    },
    missing::DecodedValue::{self, Missing1, Missing2, Normal},
};
use crate::{
    Grib2GpvUnpack,
    codetables::grib2::Table5_6,
    decoder::{
        DecodeError, Grib2SubmessageDecoder,
        simple::*,
        stream::{BitStream, NBitwiseIterator},
    },
    def::grib2::template::param_set,
};

pub(crate) struct Complex<'d>(
    pub(crate) &'d Grib2SubmessageDecoder,
    pub(crate) &'d crate::def::grib2::template::Template5_2,
);

impl<'d> Grib2GpvUnpack for Complex<'d> {
    type Iter<'a>
        = SimplePackingDecoder<ComplexPackingDecoded<'d>>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'d>, DecodeError> {
        let Self(target, template) = self;
        super::orig_field_type_is_supported(template.orig_field_type)?;

        if template.complex.group_splitting_method != 1 {
            return Err(DecodeError::NotSupported(
                "GRIB2 code table 5.4 (group splitting method)",
                template.complex.group_splitting_method.into(),
            ));
        }

        if template.complex.missing_value_management > 2 {
            return Err(DecodeError::NotSupported(
                "GRIB2 code table 5.5 (missing value management for complex packing)",
                template.complex.missing_value_management.into(),
            ));
        }

        let sect7_data = target.sect7_payload();

        let unpacked_data = decode_complex_packing(
            &template.complex,
            sect7_data,
            0,
            template.simple.num_bits,
            0,
        )?;
        let decoder = NonZeroSimplePackingDecoder::new(unpacked_data, &template.simple);
        let decoder = SimplePackingDecoder::NonZeroLength(decoder);
        Ok(decoder)
    }
}

pub(crate) struct ComplexSpatial<'d>(
    pub(crate) &'d Grib2SubmessageDecoder,
    pub(crate) &'d crate::def::grib2::template::Template5_3,
);

impl<'d> Grib2GpvUnpack for ComplexSpatial<'d> {
    type Iter<'a>
        = SimplePackingDecoder<
        SpatialDifferencingDecodeIterator<ComplexPackingDecoded<'d>, std::vec::IntoIter<i64>>,
    >
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'d>, DecodeError> {
        let Self(target, template) = self;
        super::orig_field_type_is_supported(template.orig_field_type)?;

        let spdiff_order = Table5_6::try_from(template.spatial_diff_order).map_err(|e| {
            DecodeError::NotSupported(
                "GRIB2 code table 5.6 (order of spatial differencing)",
                e.number.into(),
            )
        })?;

        if template.complex.group_splitting_method != 1 {
            return Err(DecodeError::NotSupported(
                "GRIB2 code table 5.4 (group splitting method)",
                template.complex.group_splitting_method.into(),
            ));
        }

        if template.complex.missing_value_management > 2 {
            return Err(DecodeError::NotSupported(
                "GRIB2 code table 5.5 (missing value management for complex packing)",
                template.complex.missing_value_management.into(),
            ));
        }

        let sect7_data = target.sect7_payload();
        let sect7_params = diff::SpatialDifferencingExtraDescriptors::new(
            template.spatial_diff_order,
            template.num_extra_desc_octets,
            sect7_data,
        )?;

        let unpacked_data = decode_complex_packing(
            &template.complex,
            sect7_data,
            sect7_params.len(),
            template.simple.num_bits,
            sect7_params.minimum(),
        )?;
        let first_values = sect7_params.first_values();
        let first_values = first_values.collect::<Vec<_>>().into_iter();
        let spdiff_unpacked = match spdiff_order {
            Table5_6::FirstOrderSpatialDifferencing => {
                SpatialDifferencingDecodeIterator::FirstOrder(
                    FirstOrderSpatialDifferencingDecodeIterator::new(unpacked_data, first_values),
                )
            }
            Table5_6::SecondOrderSpatialDifferencing => {
                SpatialDifferencingDecodeIterator::SecondOrder(
                    SecondOrderSpatialDifferencingDecodeIterator::new(unpacked_data, first_values),
                )
            }
            Table5_6::Missing => unreachable!(),
        };
        let decoder = NonZeroSimplePackingDecoder::new(spdiff_unpacked, &template.simple);
        let decoder = SimplePackingDecoder::NonZeroLength(decoder);
        Ok(decoder)
    }
}

pub(crate) type ComplexPackingDecoded<'d> = iter::Flatten<
    ComplexPackingValueDecodeIterator<
        Take<BitStream<&'d [u8]>>,
        Take<WithOffset<&'d [u8]>>,
        iter::Chain<Take<WithOffset<&'d [u8]>>, iter::Once<u32>>,
    >,
>;

fn decode_complex_packing<'a>(
    complex_param: &'a param_set::ComplexPacking,
    sect7_data: &'a [u8],
    sect7_offset: usize,
    num_bits: u8,
    z_min: i32,
) -> Result<ComplexPackingDecoded<'a>, DecodeError> {
    fn get_octet_length(num_bits: u8, num_groups: usize) -> Result<usize, DecodeError> {
        num_groups
            .checked_mul(usize::from(num_bits))
            .map(super::helpers::num_octets)
            .ok_or(DecodeError::LengthMismatch)
    }

    let num_groups = complex_param.num_groups as usize;
    if num_groups == 0 || num_bits > 31 {
        return Err(DecodeError::LengthMismatch);
    }
    let params_end_octet = sect7_offset;
    let group_refs_end_octet = params_end_octet
        .checked_add(get_octet_length(num_bits, num_groups)?)
        .ok_or(DecodeError::LengthMismatch)?;
    let group_widths_end_octet = group_refs_end_octet
        .checked_add(get_octet_length(
            complex_param.num_group_width_bits,
            num_groups,
        )?)
        .ok_or(DecodeError::LengthMismatch)?;
    let group_lens_end_octet = group_widths_end_octet
        .checked_add(get_octet_length(
            complex_param.num_group_len_bits,
            num_groups,
        )?)
        .ok_or(DecodeError::LengthMismatch)?;

    if group_lens_end_octet > sect7_data.len() {
        return Err(DecodeError::LengthMismatch);
    }

    let group_refs_iter = BitStream::new(
        &sect7_data[params_end_octet..group_refs_end_octet],
        usize::from(num_bits),
        num_groups,
    );
    let group_refs_iter = group_refs_iter.take(num_groups);

    let group_widths_iter = WithOffset::new(
        BitStream::new(
            &sect7_data[group_refs_end_octet..group_widths_end_octet],
            usize::from(complex_param.num_group_width_bits),
            num_groups,
        ),
        u32::from(complex_param.group_width_ref),
        1,
    )
    .take(num_groups);

    let group_lens_iter = WithOffset::new(
        BitStream::new(
            &sect7_data[group_widths_end_octet..group_lens_end_octet],
            usize::from(complex_param.num_group_len_bits),
            num_groups - 1,
        ),
        complex_param.group_len_ref,
        u32::from(complex_param.group_len_inc),
    )
    .take(num_groups - 1)
    .chain(iter::once(complex_param.group_len_last));

    Ok(ComplexPackingValueDecodeIterator::new(
        group_refs_iter,
        group_widths_iter,
        group_lens_iter,
        complex_param.missing_value_management,
        num_bits,
        z_min,
        sect7_data[group_lens_end_octet..].to_vec(),
    )
    .flatten())
}

// Types of closures with captures cannot be denoted.
// Waiting for `type_alias_impl_trait` (TAIT) getting into stable.
pub(crate) struct WithOffset<T> {
    stream: BitStream<T>,
    offset: u32,
    inc: u32,
}

impl<T> WithOffset<T> {
    fn new(stream: BitStream<T>, offset: u32, inc: u32) -> Self {
        Self {
            stream,
            offset,
            inc,
        }
    }
}

impl<T> Iterator for WithOffset<T>
where
    T: AsRef<[u8]>,
{
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().map(|v| v * self.inc + self.offset)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

#[derive(Clone)]
pub(crate) struct ComplexPackingValueDecodeIterator<I, J, K> {
    ref_iter: I,
    width_iter: J,
    length_iter: K,
    missing_value_management: u8,
    num_bits: u8,
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
        num_bits: u8,
        z_min: i32,
        data: Vec<u8>,
    ) -> Self {
        Self {
            ref_iter,
            width_iter,
            length_iter,
            missing_value_management,
            num_bits,
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
                // The specification states as follows: "For groups with a
                // constant value, associated field width is 0,
                // and no incremental data are physically present."
                let _ref = _ref.to_u32().unwrap();
                let length = length.to_usize().unwrap();
                let missing1: u32 = (1 << self.num_bits) - 1;
                // if missing1 == 0 (i.e. width == 0), missing2 should not be
                // used, so it can be set to any value.
                let missing2 = if missing1 == 0 { 0 } else { missing1 - 1 };

                if self.missing_value_management > 0 && _ref == missing1 {
                    Some(vec![Missing1; length])
                } else if self.missing_value_management == 2 && _ref == missing2 {
                    Some(vec![Missing2; length])
                } else {
                    Some(vec![Normal(_ref as i32 + self.z_min); length])
                }
            }
            (Some(_ref), Some(width), Some(length)) => {
                let (_ref, width, length) = (
                    _ref.to_u32().unwrap(),
                    width.to_usize().unwrap(),
                    length.to_usize().unwrap(),
                );
                let bits = self.start_offset_bits + width * length;
                let (pos_end, offset_bits) = (self.pos + bits / 8, bits % 8);
                let offset_byte = usize::from(offset_bits > 0);
                let missing1: u32 = (1 << width) - 1;
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
                                Normal((v + _ref) as i32 + self.z_min)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> param_set::ComplexPacking {
        param_set::ComplexPacking {
            group_splitting_method: 1,
            missing_value_management: 0,
            primary_missing_value: 0,
            secondary_missing_value: 0,
            num_groups: 1,
            group_width_ref: 0,
            num_group_width_bits: 1,
            group_len_ref: 0,
            group_len_inc: 1,
            group_len_last: 1,
            num_group_len_bits: 1,
        }
    }

    macro_rules! metadata_range_tests {
        (
            $(
                ($name:ident, $data:expr, $offset:expr, $bits:expr, $params:expr, $expected:expr $(,)?)
            ),*
            $(,)?
        ) => ($(
            #[test]
            fn $name() {
                let params = $params;
                let actual = decode_complex_packing(&params, &$data, $offset, $bits, 0);
                assert_eq!(actual.map(|_| ()), $expected);
            }
        )*);
    }

    metadata_range_tests! {
        (
            complex_packing_rejects_truncated_group_metadata,
            [0, 0],
            0,
            1,
            params(),
            Err(DecodeError::LengthMismatch),
        ),
        (metadata_exact_fit, [0; 3], 0, 1, params(), Ok(())),
        (metadata_offset_exact_fit, [0; 5], 2, 1, params(), Ok(())),
        (metadata_offset_truncated, [0; 4], 2, 1, params(), Err(DecodeError::LengthMismatch)),
        // Each of the three descriptor arrays occupies ceil(3 * 3 / 8) = 2 bytes.
        (metadata_rounds_each_array_to_octets, [0; 6], 0, 3,
            param_set::ComplexPacking {
                num_groups: 3,
                num_group_width_bits: 3,
                num_group_len_bits: 3,
                ..params()
            }, Ok(())),
        (metadata_rounded_arrays_truncated, [0; 5], 0, 3,
            param_set::ComplexPacking {
                num_groups: 3,
                num_group_width_bits: 3,
                num_group_len_bits: 3,
                ..params()
            }, Err(DecodeError::LengthMismatch)),
        // This product exceeds u32::MAX and used to wrap to zero without overflow checks.
        // On 32-bit targets the checked multiplication must instead return an error.
        (metadata_reference_bit_count_exceeds_u32, [0; 0], 0, 2,
            param_set::ComplexPacking {
                num_groups: 1 << 31,
                num_group_width_bits: 0,
                num_group_len_bits: 0,
                ..params()
            }, Err(DecodeError::LengthMismatch)),
        (metadata_width_bit_count_exceeds_u32, [0; 0], 0, 0,
            param_set::ComplexPacking {
                num_groups: 1 << 31,
                num_group_width_bits: 2,
                num_group_len_bits: 0,
                ..params()
            }, Err(DecodeError::LengthMismatch)),
        (metadata_length_bit_count_exceeds_u32, [0; 0], 0, 0,
            param_set::ComplexPacking {
                num_groups: 1 << 31,
                num_group_width_bits: 0,
                num_group_len_bits: 2,
                ..params()
            }, Err(DecodeError::LengthMismatch)),
        (metadata_reference_end_overflows, [0; 0], usize::MAX, 1,
            params(), Err(DecodeError::LengthMismatch)),
        (metadata_width_end_overflows, [0; 0], usize::MAX - 1, 1,
            params(), Err(DecodeError::LengthMismatch)),
        (metadata_length_end_overflows, [0; 0], usize::MAX - 2, 1,
            params(), Err(DecodeError::LengthMismatch)),
    }

    macro_rules! reference_width_tests {
        (
            $(
                ($name:ident, $data:expr, $bits:expr, $groups:expr, $last_length:expr, $expected:expr $(,)?)
            ),*
            $(,)?
        ) => ($(
            #[test]
            fn $name() {
                // Constant groups need no packed values. Descriptor widths of
                // zero isolate reference-width and group-count validation.
                let params = param_set::ComplexPacking {
                    num_groups: $groups,
                    num_group_width_bits: 0,
                    group_len_ref: 1,
                    group_len_last: $last_length,
                    num_group_len_bits: 0,
                    ..params()
                };
                let actual = decode_complex_packing(&params, &$data, 0, $bits, 0)
                    .map(|values| values.collect::<Vec<_>>());
                assert_eq!(actual, $expected);
            }
        )*);
    }

    reference_width_tests! {
        (
            complex_packing_accepts_zero_reference_width,
            [],
            0,
            1,
            1,
            Ok(vec![Normal(0)]),
        ),
        (
            complex_packing_accepts_31_bit_reference_width,
            // The 31-bit reference is i32::MAX, followed by one padding bit.
            // With missing-value management disabled it is a normal value.
            [0xff, 0xff, 0xff, 0xfe],
            31,
            1,
            1,
            Ok(vec![Normal(i32::MAX)]),
        ),
        (
            complex_packing_reads_multiple_group_references,
            // One-bit references 0 and 1, followed by six padding bits.
            // The first group has length 1; the last uses its explicit length 2.
            [0x40],
            1,
            2,
            2,
            Ok(vec![Normal(0), Normal(1), Normal(1)]),
        ),
        (
            complex_packing_rejects_maximum_reference_width,
            // Enough bytes for the declared reference: rejection must not
            // depend on truncated metadata.
            [0; 32],
            u8::MAX,
            1,
            1,
            Err(DecodeError::LengthMismatch),
        ),
    }

    #[test]
    fn dispatch_rejects_truncated_complex_metadata() -> Result<(), Box<dyn std::error::Error>> {
        // Section 5: template 5.2, one point and one group. Each descriptor
        // array needs one byte (one entry encoded with one bit).
        let mut sect5 = vec![0; 47];
        sect5[..4].copy_from_slice(&47u32.to_be_bytes());
        sect5[4] = 5;
        sect5[5..9].copy_from_slice(&1u32.to_be_bytes());
        sect5[9..11].copy_from_slice(&2u16.to_be_bytes());
        sect5[19] = 1; // Bits per group reference.
        sect5[21] = 1; // General group splitting.
        sect5[31..35].copy_from_slice(&1u32.to_be_bytes());
        sect5[36] = 1; // Bits per group width.
        sect5[41] = 1; // Group length increment.
        sect5[42..46].copy_from_slice(&1u32.to_be_bytes());
        sect5[46] = 1; // Bits per scaled group length.
        let sect6 = vec![0, 0, 0, 6, 6, 255]; // No bitmap.
        let sect7 = vec![0, 0, 0, 7, 7, 0, 0]; // Only two descriptor bytes.
        let decoder = Grib2SubmessageDecoder::new(1, sect5, sect6, sect7)?;

        assert!(matches!(
            decoder.dispatch(),
            Err(crate::GribError::DecodeError(DecodeError::LengthMismatch))
        ));
        Ok(())
    }

    #[test]
    fn complex_packing_rejects_zero_groups() {
        let mut params = params();
        params.num_groups = 0;

        assert!(matches!(
            decode_complex_packing(&params, &[], 0, 1, 0),
            Err(DecodeError::LengthMismatch)
        ));
    }

    #[test]
    fn complex_packing_rejects_reference_widths_that_overflow_value_markers() {
        let mut params = params();
        params.num_group_width_bits = 0;

        assert!(matches!(
            decode_complex_packing(&params, &[0, 0, 0, 0, 0], 0, 32, 0),
            Err(DecodeError::LengthMismatch)
        ));
    }
}
