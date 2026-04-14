use grib_template_helpers::WriteToBuffer;

use crate::{
    WriteGrib2DataSections,
    def::grib2::template::param_set::SimplePacking,
    encoder::{Encode, helpers::BitsRequired, writer},
};

/// Strategies applied when performing simple packing on numerical sequences.
/// Simple packing is a method for discretizing continuous numerical values as
/// integers, and various approaches can be taken during this process.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SimplePackingStrategy {
    /// A strategy specifying how many decimal places to consider valid for the
    /// numbers. This strategy is effective for various types of data, such as
    /// observation data obtained from specific observation instruments, where
    /// precision is clearly defined.
    Decimal(i16),
}

pub(crate) struct SimplePackingEncoder<'a> {
    data: &'a [f64],
    strategy: SimplePackingStrategy,
}

impl<'a> SimplePackingEncoder<'a> {
    pub(crate) fn new(data: &'a [f64], strategy: SimplePackingStrategy) -> Self {
        Self { data, strategy }
    }
}

impl<'a> Encode for SimplePackingEncoder<'a> {
    type Output = SimplePackingEncoded;

    fn encode(&self) -> Self::Output {
        match self.strategy {
            SimplePackingStrategy::Decimal(dec) => {
                let (params, scaled) = determine_simple_packing_params(self.data, dec);
                let coded = if params.num_bits == 0 {
                    CodedValues::Unique(self.data.len())
                } else {
                    let exp = 2_f64.powf(params.exp as f64);
                    let coded = scaled
                        .iter()
                        .map(|value| ((value - params.ref_val as f64) / exp).round() as u32)
                        .collect::<Vec<_>>();
                    CodedValues::NonUnique(coded)
                };
                SimplePackingEncoded::new(params, coded)
            }
        }
    }
}

pub(crate) struct SimplePackingEncoded {
    params: SimplePacking,
    coded: CodedValues,
}

impl SimplePackingEncoded {
    fn new(params: SimplePacking, coded: CodedValues) -> Self {
        Self { params, coded }
    }

    pub(crate) fn params(&self) -> &SimplePacking {
        &self.params
    }
}

impl WriteGrib2DataSections for SimplePackingEncoded {
    fn section5_len(&self) -> usize {
        21
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
        pos += 0_u16.write_to_buffer(&mut buf[pos..])?; // payload.template_num
        pos += self.params.write_to_buffer(&mut buf[pos..])?;
        pos += 0_u8.write_to_buffer(&mut buf[pos..])?; // payload.template.orig_field_type

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
            CodedValues::NonUnique(vec) => {
                let nbitwise = writer::NBitwise::new(&vec, self.params.num_bits as usize);
                nbitwise.num_bytes_required()
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
            CodedValues::NonUnique(vec) => {
                let nbitwise = writer::NBitwise::new(&vec, self.params.num_bits as usize);
                pos += nbitwise.write_to_buffer(&mut buf[pos..])?;
            }
            CodedValues::Unique(_) => {}
        }

        Ok(pos)
    }
}

pub(crate) fn determine_simple_packing_params(
    values: &[f64],
    dec: i16,
) -> (SimplePacking, Vec<f64>) {
    let mut min = f64::MAX;
    let mut max = f64::MIN;
    let scaled = values
        .iter()
        .map(|value| {
            let scaled = value * 10_f64.powf(dec as f64);
            (min, max) = (scaled.min(min), scaled.max(max));
            scaled
        })
        .collect::<Vec<_>>();
    let ref_val = min as f32;
    if min == max {
        let params = SimplePacking {
            ref_val,
            exp: 0,
            dec,
            num_bits: 0,
        };
        (params, Vec::new())
    } else {
        let range = max - min;
        let exp = 0;
        let num_bits = range.bits_required();
        // TODO: if `num_bits` is too large, increase `exp` to reduce `num_bits`.
        let params = SimplePacking {
            ref_val,
            exp,
            dec,
            num_bits,
        };
        (params, scaled)
    }
}

enum CodedValues {
    NonUnique(Vec<u32>),
    Unique(usize),
}

impl CodedValues {
    pub(crate) fn num_values(&self) -> usize {
        match self {
            Self::NonUnique(vec) => vec.len(),
            Self::Unique(size) => *size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_decimal_strategy {
        ($((
            $name:ident,
            $input:expr,
            $decimal:expr,
            $expected_params:expr,
        ),)*) => ($(
            #[test]
            fn $name() {
                let values = $input;
                let encoder =
                    SimplePackingEncoder::new(&values, SimplePackingStrategy::Decimal($decimal));
                let encoded = encoder.encode();
                let actual_params = encoded.params();
                let expected_params = $expected_params;
                assert_eq!(actual_params, &expected_params);
                let actual_num_values = encoded.coded.num_values();
                assert_eq!(actual_num_values, values.len());
            }
        )*);
    }

    test_decimal_strategy! {
        (
            decimal_strategy_with_decimal_0,
            (2..11).map(|val| val as f64).collect::<Vec<_>>(),
            0,
            SimplePacking {
                ref_val: 2.,
                exp: 0,
                dec: 0,
                num_bits: 4,
            },
        ),
        (
            decimal_strategy_with_decimal_1,
            (2..11).map(|val| val as f64).collect::<Vec<_>>(),
            1,
            SimplePacking {
                ref_val: 20.,
                exp: 0,
                dec: 1,
                num_bits: 7,
            },
        ),
        (
            decimal_strategy_with_decimal_0_for_unique_values,
            vec![10.0_f64; 256],
            0,
            SimplePacking {
                ref_val: 10.0,
                exp: 0,
                dec: 0,
                num_bits: 0,
            },
        ),
    }

    macro_rules! grib2_coded_values_roundtrip_tests {
        ($(($name:ident, $input:expr, $decimal:expr),)*) => ($(
            #[test]
            fn $name() -> Result<(), Box<dyn std::error::Error>> {
                let values = $input;
                let encoder =
                    SimplePackingEncoder::new(&values, SimplePackingStrategy::Decimal($decimal));
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
            grib2_coded_values_roundtrip_test_with_decimal_0_and_nonunique_values,
            (2..11).map(|val| val as f64).collect::<Vec<_>>(),
            0
        ),
        (
            grib2_coded_values_roundtrip_test_with_decimal_0_and_unique_values,
            vec![10.0_f64; 256],
            0
        ),
    }
}
