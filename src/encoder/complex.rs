use grib_template_helpers::WriteToBuffer;

use crate::{
    SimplePackingStrategy, WriteGrib2DataSections,
    def::grib2::template::param_set::{ComplexPacking, SimplePacking},
    encoder::{Encode, helpers::BitsRequired, writer},
};

/// Strategies applied when performing complex packing on numerical sequences.
/// Complex packing is a method that divides a sequence of numbers into groups
/// and efficiently compresses each group to improve the overall compression
/// ratio of the data.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ComplexPackingStrategy {
    /// A strategy that pre-reads a specified number of elements to determine
    /// whether to add an element to the current group.
    LookAhead(usize),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpatialDifferencingOption {
    None,
}

pub(crate) struct Encoder<'a> {
    data: &'a [f64],
    simple_packing_strategy: SimplePackingStrategy,
    complex_packing_strategy: ComplexPackingStrategy,
}

impl<'a> Encoder<'a> {
    pub(crate) fn new(
        data: &'a [f64],
        simple_packing_strategy: SimplePackingStrategy,
        complex_packing_strategy: ComplexPackingStrategy,
    ) -> Self {
        Self {
            data,
            simple_packing_strategy,
            complex_packing_strategy,
        }
    }
}

impl<'a> Encode for Encoder<'a> {
    type Output = Encoded;

    fn encode(&self) -> Self::Output {
        match self.complex_packing_strategy {
            ComplexPackingStrategy::LookAhead(num) => {
                let (mut simple, scaled) = match self.simple_packing_strategy {
                    SimplePackingStrategy::Decimal(decimal) => {
                        super::determine_simple_packing_params(self.data, decimal)
                    }
                };
                let (complex, coded) = if simple.num_bits == 0 {
                    let len = self.data.len();
                    (
                        ComplexPacking::for_unique_values(len),
                        CodedValues::Unique(len),
                    )
                } else {
                    let exp = 2_f64.powf(simple.exp as f64);
                    let integers = scaled
                        .iter()
                        .map(|value| ((value - simple.ref_val as f64) / exp).round() as u32)
                        .collect::<Vec<_>>();
                    let num_bits = integers.iter().max().unwrap().bits_required();
                    simple.num_bits = num_bits;
                    let groups = Groups::from_values(&integers, num);
                    (
                        ComplexPacking::from(&groups),
                        CodedValues::NonUnique(groups),
                    )
                };
                Encoded::new(simple, complex, coded)
            }
        }
    }
}

impl ComplexPacking {
    fn for_unique_values(len: usize) -> Self {
        let len = len as u32;
        Self {
            group_splitting_method: 1,
            missing_value_management: 0,
            primary_missing_value: 0xffffffff,
            secondary_missing_value: 0xffffffff,
            num_groups: 1,
            group_width_ref: 0,
            num_group_width_bits: 0,
            group_len_ref: len,
            group_len_inc: 1,
            group_len_last: len,
            num_group_len_bits: 0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Encoded {
    simple: SimplePacking,
    complex: ComplexPacking,
    coded: CodedValues,
}

impl Encoded {
    fn new(simple: SimplePacking, complex: ComplexPacking, coded: CodedValues) -> Self {
        Self {
            simple,
            complex,
            coded,
        }
    }

    pub(crate) fn params(&self) -> (&SimplePacking, &ComplexPacking) {
        (&self.simple, &self.complex)
    }
}

impl WriteGrib2DataSections for Encoded {
    fn section5_len(&self) -> usize {
        47
    }

    fn write_section5(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let len = self.section5_len();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        let mut pos = 0;
        pos += (len as u32).write_to_buffer(&mut buf[pos..])?; // header.len
        pos += 5_u8.write_to_buffer(&mut buf[pos..])?; // header.sect_num
        pos += (self.coded.num_values() as u32).write_to_buffer(&mut buf[pos..])?; // payload.num_encoded_points
        pos += 2_u16.write_to_buffer(&mut buf[pos..])?; // payload.template_num
        pos += self.simple.write_to_buffer(&mut buf[pos..])?;
        pos += 0_u8.write_to_buffer(&mut buf[pos..])?; // payload.template.orig_field_type
        pos += self.complex.write_to_buffer(&mut buf[pos..])?;

        Ok(pos)
    }

    fn section6_len(&self) -> usize {
        6
    }

    fn write_section6(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let len = self.section6_len();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        let mut pos = 0;
        pos += (len as u32).write_to_buffer(&mut buf[pos..])?;
        pos += 6_u8.write_to_buffer(&mut buf[pos..])?;
        pos += 255_u8.write_to_buffer(&mut buf[pos..])?;

        Ok(pos)
    }

    fn section7_len(&self) -> usize {
        let len = match &self.coded {
            CodedValues::NonUnique(Groups(inner)) => {
                let num_groups = self.complex.num_groups as usize;
                let bits_refs = self.simple.num_bits as usize * num_groups;
                let bits_widths = self.complex.num_group_width_bits as usize * num_groups;
                let bits_lengths = self.complex.num_group_len_bits as usize * num_groups;
                let octets_values: usize = inner
                    .iter()
                    .map(|g| (g.len() * g.width as usize).div_ceil(8))
                    .sum();
                bits_refs.div_ceil(8)
                    + bits_widths.div_ceil(8)
                    + bits_lengths.div_ceil(8)
                    + octets_values
            }
            CodedValues::Unique(_) => 0,
        };
        5 + len
    }

    fn write_section7(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let len = self.section7_len();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        let mut pos = 0;
        pos += (len as u32).write_to_buffer(&mut buf[pos..])?;
        pos += 7_u8.write_to_buffer(&mut buf[pos..])?;
        match &self.coded {
            CodedValues::NonUnique(Groups(inner)) => {
                if self.simple.num_bits != 0 {
                    let refs = inner.iter().map(|g| g.ref_val).collect::<Vec<_>>();
                    let nbitwise = writer::NBitwise::new(&refs, self.simple.num_bits as usize);
                    pos += nbitwise.write_to_buffer(&mut buf[pos..])?;
                }

                if self.complex.num_group_width_bits != 0 {
                    let widths = inner
                        .iter()
                        .map(|g| (g.width - self.complex.group_width_ref) as u32)
                        .collect::<Vec<_>>();
                    let nbitwise =
                        writer::NBitwise::new(&widths, self.complex.num_group_width_bits as usize);
                    pos += nbitwise.write_to_buffer(&mut buf[pos..])?;
                }

                if self.complex.group_len_inc != 0 && self.complex.num_group_len_bits != 0 {
                    let lengths = inner
                        .iter()
                        .take(self.complex.num_groups as usize - 1)
                        .map(|g| {
                            (g.len() as u32 - self.complex.group_len_ref)
                                / self.complex.group_len_inc as u32
                        })
                        .chain(std::iter::once(0))
                        .collect::<Vec<_>>();
                    let nbitwise =
                        writer::NBitwise::new(&lengths, self.complex.num_group_len_bits as usize);
                    pos += nbitwise.write_to_buffer(&mut buf[pos..])?;
                }

                for group in inner {
                    if group.width != 0 {
                        let nbitwise = writer::NBitwise::new(&group.values, group.width as usize);
                        pos += nbitwise.write_to_buffer(&mut buf[pos..])?;
                    }
                }
            }
            CodedValues::Unique(_) => {}
        }

        Ok(pos)
    }
}

#[derive(Debug)]
enum CodedValues {
    NonUnique(Groups),
    Unique(usize),
}

impl CodedValues {
    pub(crate) fn num_values(&self) -> usize {
        match self {
            Self::NonUnique(vec) => vec.num_values(),
            Self::Unique(size) => *size,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Groups(Vec<Group>);

impl Groups {
    fn from_values(values: &[u32], num_lookahead: usize) -> Self {
        let mut groups = Vec::new();
        let mut start = 0;

        while start < values.len() {
            let mut end = start + 1;

            let v = values[start];
            let (mut min, mut max) = (v, v);
            let mut width = 0;

            while end < values.len() {
                let v = values[end];
                let (new_min, new_max) = (min.min(v), max.max(v));
                let new_width = (new_max - new_min).bits_required();

                let len = end - start;
                let cost_extend = group_cost(len + 1, new_width);
                let cost_keep = group_cost(len, width)
                    + new_group_cost_estimated(&values[end..], num_lookahead);
                if cost_keep < cost_extend {
                    break;
                }

                min = new_min;
                max = new_max;
                width = new_width;
                end += 1;
            }

            groups.push(Group::from_values(&values[start..end]));
            start = end;
        }

        Self(groups)
    }

    fn optimal_length_params(&self) -> Option<OptimalLengthParams> {
        let Self(inner) = self;
        let num_lengths = inner.len() - 1; // the last group is treated separately
        if num_lengths == 0 {
            None
        } else {
            let lengths = inner
                .iter()
                .take(num_lengths)
                .map(|g| g.values.len())
                .collect::<Vec<_>>();
            Some(OptimalLengthParams::from(&lengths[..]))
        }
    }

    fn num_values(&self) -> usize {
        let Self(inner) = self;
        inner.iter().map(|g| g.values.len()).sum()
    }
}

fn group_cost(len: usize, width: u8) -> usize {
    len * width as usize
}

fn new_group_cost_estimated(values: &[u32], num_lookahead: usize) -> usize {
    if values.is_empty() {
        return 0;
    }

    let lookahead = values.iter().take(num_lookahead);

    let (mut min, mut max) = (u32::MAX, u32::MIN);
    let mut len = 0;

    for &v in lookahead {
        (min, max) = (min.min(v), max.max(v));
        len += 1;
    }
    let width = (max - min).bits_required();
    group_cost(len, width)
}

impl From<&Groups> for ComplexPacking {
    fn from(value: &Groups) -> Self {
        let Groups(inner) = value;
        let group_width_ref = inner.iter().map(|g| g.width).min().unwrap();
        let max_width = inner.iter().map(|g| g.width).max().unwrap();
        let num_group_width_bits = (max_width - group_width_ref).bits_required();
        let (group_len_ref, group_len_inc, num_group_len_bits) =
            if let Some(length_params) = value.optimal_length_params() {
                (
                    length_params.ref_ as u32,
                    length_params.inc as u8,
                    length_params.num_bits,
                )
            } else {
                (0, 0, 0)
            };
        Self {
            group_splitting_method: 1,
            missing_value_management: 0,
            primary_missing_value: 0xffffffff,
            secondary_missing_value: 0xffffffff,
            num_groups: inner.len() as u32,
            group_width_ref,
            num_group_width_bits,
            group_len_ref,
            group_len_inc,
            group_len_last: inner.last().unwrap().values.len() as u32,
            num_group_len_bits,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Group {
    pub ref_val: u32,
    pub width: u8,
    pub values: Vec<u32>,
}

impl Group {
    fn from_values(values: &[u32]) -> Self {
        let ref_val = *values.iter().min().unwrap();
        let mut max_diff = u32::MIN;
        let diffs = values
            .iter()
            .map(|v| {
                let diff = v - ref_val;
                max_diff = max_diff.max(diff);
                diff
            })
            .collect();
        let width = max_diff.bits_required();

        Group {
            ref_val,
            width,
            values: diffs,
        }
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, PartialEq, Eq)]
struct OptimalLengthParams {
    ref_: usize,
    inc: usize,
    num_bits: u8,
    total_bits: usize,
}

impl OptimalLengthParams {
    fn new(ref_: usize, inc: usize, num_bits: u8, total_bits: usize) -> Self {
        Self {
            ref_,
            inc,
            num_bits,
            total_bits,
        }
    }
}

impl From<&[usize]> for OptimalLengthParams {
    fn from(value: &[usize]) -> Self {
        let ref_ = value.iter().min().unwrap();
        let diffs = value.iter().map(|&l| l - ref_).collect::<Vec<_>>();
        let gcd_ = diffs.iter().copied().reduce(gcd).unwrap_or(0);

        if gcd_ == 0 {
            return Self::new(*ref_, 0, 0, 0);
        }

        let max_code = diffs.iter().map(|d| d / gcd_).max().unwrap();
        let num_bits = max_code.bits_required();
        let total_bits = num_bits as usize * value.len();

        Self::new(*ref_, gcd_, num_bits, total_bits)
    }
}

fn gcd(m: usize, n: usize) -> usize {
    let (m, n) = if m > n { (m, n) } else { (n, m) };
    if n == 0 { m } else { gcd(n, m % n) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping() {
        let mut values = (0_u32..24).collect::<Vec<_>>();
        values[10] = 64;
        values[21] = 128;
        values[22] = 256;
        let actual = Groups::from_values(&values, 4);
        let expected = Groups(vec![
            Group {
                ref_val: 0,
                width: 4,
                values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
            Group {
                ref_val: 64,
                width: 0,
                values: vec![0],
            },
            Group {
                ref_val: 11,
                width: 4,
                values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            },
            Group {
                ref_val: 128,
                width: 8,
                values: vec![0, 128],
            },
            Group {
                ref_val: 23,
                width: 0,
                values: vec![0],
            },
        ]);
        assert_eq!(actual, expected);
    }

    macro_rules! test_optimal_length_params {
        ($((
            $name:ident,
            $input:expr,
            $expected:expr,
        ),)*) => ($(
            #[test]
            fn $name() {
                let lengths = $input;
                let actual = OptimalLengthParams::from(&lengths[..]);
                let expected = $expected;
                assert_eq!(actual, expected);
            }
        )*);
    }

    test_optimal_length_params! {
        (
            optimal_length_params_for_all_zero,
            vec![0, 0, 0, 0, 0],
            OptimalLengthParams::new(0, 0, 0, 0),
        ),
        (
            optimal_length_params_for_all_same_nonzero,
            vec![5, 5, 5, 5, 5],
            OptimalLengthParams::new(5, 0, 0, 0),
        ),
        (
            optimal_length_params_for_gcd_being_one,
            vec![26, 24, 20, 14, 13],
            OptimalLengthParams::new(13, 1, 4, 20),
        ),
        (
            optimal_length_params_for_gcd_being_integer_other_than_zero_and_one,
            vec![13, 19, 22, 16, 25],
            OptimalLengthParams::new(13, 3, 3, 15),
        ),
    }

    macro_rules! grib2_coded_values_roundtrip_tests {
        ($(($name:ident, $input:expr),)*) => ($(
            #[test]
            fn $name() -> Result<(), Box<dyn std::error::Error>> {
                let values = $input;
                let encoder = Encoder::new(
                    &values,
                    SimplePackingStrategy::Decimal(0),
                    ComplexPackingStrategy::LookAhead(4),
                );
                let encoded = encoder.encode();
                let mut sect5 = vec![0; encoded.section5_len()];
                encoded.write_section5(&mut sect5)?;
                let mut sect6 = vec![0; encoded.section6_len()];
                encoded.write_section6(&mut sect6)?;
                let mut sect7 = vec![0; encoded.section7_len()];
                encoded.write_section7(&mut sect7)?;
                let decoder = crate::Grib2SubmessageDecoder::new(values.len(), sect5, sect6, sect7)?;
                let actual = decoder.dispatch()?.collect::<Vec<_>>();
                let expected = values.iter().map(|val| *val as f32).collect::<Vec<_>>();
                assert_eq!(actual, expected);
                Ok(())
            }
        )*);
    }

    grib2_coded_values_roundtrip_tests! {
        (
            grib2_coded_values_roundtrip_test_with_nonunique_values,
            (2..11).map(|val| val as f64).collect::<Vec<_>>()
        ),
        (
            grib2_coded_values_roundtrip_test_with_nonunique_values_and_group_len_inc_being_0,
            (2..5).map(|val| val as f64).collect::<Vec<_>>()
        ),
        (
            grib2_coded_values_roundtrip_test_with_unique_values,
            vec![10.0_f64; 256]
        ),
        (
            grib2_coded_values_roundtrip_test_with_zero_only_groups,
            vec![0, 0, 0, 100, 10, 2, 2, 1]
                .into_iter()
                .flat_map(|val| [val as f64; 8])
                .collect::<Vec<_>>()
        ),
    }
}
