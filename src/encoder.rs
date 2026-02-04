use crate::{def::grib2::template::param_set::SimplePacking, encoder::writer::WriteToBuffer};

pub enum SimplePackingStrategy {
    Decimal(i16),
}

pub struct SimplePackingEncoder<'a> {
    data: &'a [f64],
    strategy: SimplePackingStrategy,
}

impl<'a> SimplePackingEncoder<'a> {
    pub fn new(data: &'a [f64], strategy: SimplePackingStrategy) -> Self {
        Self { data, strategy }
    }

    pub fn encode(&self) -> SimplePackingEncoded {
        match self.strategy {
            SimplePackingStrategy::Decimal(dec) => {
                let (params, scaled) = determine_simple_packing_params(self.data, dec);
                let exp = 2_f64.powf(params.exp as f64);
                let coded = scaled
                    .iter()
                    .map(|value| ((value - params.ref_val as f64) / exp).round() as u32)
                    .collect::<Vec<_>>();
                SimplePackingEncoded::new(params, coded)
            }
        }
    }
}

pub struct SimplePackingEncoded {
    params: SimplePacking,
    coded: Vec<u32>,
}

impl SimplePackingEncoded {
    fn new(params: SimplePacking, coded: Vec<u32>) -> Self {
        Self { params, coded }
    }

    pub fn params(&self) -> &SimplePacking {
        &self.params
    }

    pub fn section5_size(&self) -> usize {
        21
    }

    pub fn write_section5(&self, buf: &mut [u8]) -> Result<(), &'static str> {
        let size = self.section5_size();
        if buf.len() < size {
            return Err("destination buffer is too small");
        }

        (size as u32).write_to_buffer(&mut buf[0..4])?; // header.len
        5_u8.write_to_buffer(&mut buf[4..5])?; // header.sect_num
        (self.coded.len() as u32).write_to_buffer(&mut buf[5..9])?; // payload.num_encoded_points
        0_u16.write_to_buffer(&mut buf[9..11])?; // payload.template_num
        self.params.ref_val.write_to_buffer(&mut buf[11..15])?;
        self.params.exp.write_to_buffer(&mut buf[15..17])?;
        self.params.dec.write_to_buffer(&mut buf[17..19])?;
        self.params.num_bits.write_to_buffer(&mut buf[19..20])?;
        0_u8.write_to_buffer(&mut buf[20..21])?; // payload.template.orig_field_type

        Ok(())
    }

    pub fn section6_size(&self) -> usize {
        6
    }

    pub fn write_section6(&self, buf: &mut [u8]) -> Result<(), &'static str> {
        let size = self.section6_size();
        if buf.len() < size {
            return Err("destination buffer is too small");
        }

        (size as u32).write_to_buffer(&mut buf[0..4])?;
        6_u8.write_to_buffer(&mut buf[4..5])?;
        255_u8.write_to_buffer(&mut buf[5..6])?;

        Ok(())
    }

    pub fn section7_size(&self) -> usize {
        let nbitwise = writer::NBitwise::new(&self.coded, self.params.num_bits as usize);
        nbitwise.num_bytes_required() + 5
    }

    pub fn write_section7(&self, buf: &mut [u8]) -> Result<(), &'static str> {
        let nbitwise = writer::NBitwise::new(&self.coded, self.params.num_bits as usize);
        let size = self.section7_size();
        if buf.len() < size {
            return Err("destination buffer is too small");
        }

        (size as u32).write_to_buffer(&mut buf[0..4])?;
        7_u8.write_to_buffer(&mut buf[4..5])?;
        nbitwise.write_to_buffer(&mut buf[5..size])?;

        Ok(())
    }
}

fn determine_simple_packing_params(values: &[f64], dec: i16) -> (SimplePacking, Vec<f64>) {
    let mut min = 0.;
    let mut max = 0.;
    let scaled = values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let scaled = value * 10_f64.powf(dec as f64);
            (min, max) = if i == 0 {
                (scaled, scaled)
            } else {
                (scaled.min(min), scaled.max(max))
            };
            scaled
        })
        .collect::<Vec<_>>();
    let ref_val = min as f32;
    let range = max - min;
    let exp = 0;
    let num_bits = (range + 1.).log2().ceil() as u8;
    // TODO: if `num_bits` is too large, increase `exp` to reduce `num_bits`.
    let params = SimplePacking {
        ref_val,
        exp,
        dec,
        num_bits,
    };
    (params, scaled)
}

mod to_grib_signed;
mod writer;

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_decimal_strategy{
        ($(($name:ident, $decimal:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let values = (2..11).map(|val| val as f64).collect::<Vec<_>>();
                let encoder =
                    SimplePackingEncoder::new(&values, SimplePackingStrategy::Decimal($decimal));
                let encoded = encoder.encode();
                let actual = encoded.params();
                let expected = $expected;
                assert_eq!(actual, &expected);
            }
        )*);
    }

    test_decimal_strategy! {
        (
            decimal_strategy_with_decimal_0,
            0,
            SimplePacking {
                ref_val: 2.,
                exp: 0,
                dec: 0,
                num_bits: 4,
            }
        ),
        (
            decimal_strategy_with_decimal_1,
            1,
            SimplePacking {
                ref_val: 20.,
                exp: 0,
                dec: 1,
                num_bits: 7,
            }
        ),
    }

    #[test]
    fn grib2_writing() -> Result<(), Box<dyn std::error::Error>> {
        let values = (2..11).map(|val| val as f64).collect::<Vec<_>>();
        let encoder = SimplePackingEncoder::new(&values, SimplePackingStrategy::Decimal(0));
        let encoded = encoder.encode();
        let mut sect5 = vec![0; encoded.section5_size()];
        encoded.write_section5(&mut sect5)?;
        let mut sect6 = vec![0; encoded.section6_size()];
        encoded.write_section6(&mut sect6)?;
        let mut sect7 = vec![0; encoded.section7_size()];
        encoded.write_section7(&mut sect7)?;
        let decoder = crate::Grib2SubmessageDecoder::new(values.len(), sect5, sect6, sect7)?;
        let actual = decoder.dispatch()?.collect::<Vec<_>>();
        let expected = values.iter().map(|val| *val as f32).collect::<Vec<_>>();
        assert_eq!(actual, expected);
        Ok(())
    }
}
