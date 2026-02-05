use crate::def::grib2::template::param_set::SimplePacking;
pub use crate::encoder::writer::WriteToBuffer;

/// A serializer that writes the byte sequence of sections concerning GPV data
/// to the output buffer
pub trait WriteGrib2DataSections {
    /// Returns the length of the byte sequence in Section 5.
    fn section5_len(&self) -> usize;

    /// Writes the byte sequence of Section 5 to the output buffer
    fn write_section5(&self, buf: &mut [u8]) -> Result<(), &'static str>;

    /// Returns the length of the byte sequence in Section 6.
    fn section6_len(&self) -> usize;

    /// Writes the byte sequence of Section 6 to the output buffer
    fn write_section6(&self, buf: &mut [u8]) -> Result<(), &'static str>;

    /// Returns the length of the byte sequence in Section 7.
    fn section7_len(&self) -> usize;

    /// Writes the byte sequence of Section 7 to the output buffer
    fn write_section7(&self, buf: &mut [u8]) -> Result<(), &'static str>;
}

/// Strategies applied when performing simple packing on numerical sequences.
/// Simple packing is a method for discretizing continuous numerical values as
/// integers, and various approaches can be taken during this process.
pub enum SimplePackingStrategy {
    /// A strategy specifying how many decimal places to consider valid for the
    /// numbers. This strategy is effective for various types of data, such as
    /// observation data obtained from specific observation instruments, where
    /// precision is clearly defined.
    Decimal(i16),
}

/// Simple packing encoder.
pub struct SimplePackingEncoder<'a> {
    data: &'a [f64],
    strategy: SimplePackingStrategy,
}

impl<'a> SimplePackingEncoder<'a> {
    pub fn new(data: &'a [f64], strategy: SimplePackingStrategy) -> Self {
        Self { data, strategy }
    }

    /// Performs data encoding.
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

/// Data obtained through encoding using simple packing. Instances are typically
/// used to write GRIB2 data via the methods defined in
/// [`WriteGrib2DataSections`].
pub struct SimplePackingEncoded {
    params: SimplePacking,
    coded: Vec<u32>,
}

impl SimplePackingEncoded {
    fn new(params: SimplePacking, coded: Vec<u32>) -> Self {
        Self { params, coded }
    }

    /// Returns the parameter set for simple packing.
    pub fn params(&self) -> &SimplePacking {
        &self.params
    }
}

impl WriteGrib2DataSections for SimplePackingEncoded {
    fn section5_len(&self) -> usize {
        21
    }

    fn write_section5(&self, buf: &mut [u8]) -> Result<(), &'static str> {
        let len = self.section5_len();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        (len as u32).write_to_buffer(&mut buf[0..4])?; // header.len
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

    fn section6_len(&self) -> usize {
        6
    }

    fn write_section6(&self, buf: &mut [u8]) -> Result<(), &'static str> {
        let len = self.section6_len();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        (len as u32).write_to_buffer(&mut buf[0..4])?;
        6_u8.write_to_buffer(&mut buf[4..5])?;
        255_u8.write_to_buffer(&mut buf[5..6])?;

        Ok(())
    }

    fn section7_len(&self) -> usize {
        let nbitwise = writer::NBitwise::new(&self.coded, self.params.num_bits as usize);
        nbitwise.num_bytes_required() + 5
    }

    fn write_section7(&self, buf: &mut [u8]) -> Result<(), &'static str> {
        let nbitwise = writer::NBitwise::new(&self.coded, self.params.num_bits as usize);
        let len = self.section7_len();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        (len as u32).write_to_buffer(&mut buf[0..4])?;
        7_u8.write_to_buffer(&mut buf[4..5])?;
        nbitwise.write_to_buffer(&mut buf[5..len])?;

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

pub fn write_section0(discipline: u8, len: usize, buf: &mut [u8]) -> Result<(), &'static str> {
    const HEAD: [u8; 6] = [0x47, 0x52, 0x49, 0x42, 0xff, 0xff];
    const EDITION: u8 = 2;
    const LEN: usize = 16;
    if buf.len() < LEN {
        return Err("destination buffer is too small");
    }

    HEAD.write_to_buffer(&mut buf[0..HEAD.len()])?;
    discipline.write_to_buffer(&mut buf[6..7])?;
    EDITION.write_to_buffer(&mut buf[7..8])?;
    (len as u64).write_to_buffer(&mut buf[8..16])?;
    Ok(())
}

pub fn write_section1(
    payload: &crate::def::grib2::Section1Payload,
    buf: &mut [u8],
) -> Result<(), &'static str> {
    const LEN: usize = 0x15;
    if buf.len() < LEN {
        return Err("destination buffer is too small");
    }

    (LEN as u32).write_to_buffer(&mut buf[0..4])?;
    2_u8.write_to_buffer(&mut buf[4..5])?;
    payload.centre_id.write_to_buffer(&mut buf[5..7])?;
    payload.subcentre_id.write_to_buffer(&mut buf[7..9])?;
    payload
        .master_table_version
        .write_to_buffer(&mut buf[9..10])?;
    payload
        .local_table_version
        .write_to_buffer(&mut buf[10..11])?;
    payload
        .ref_time_significance
        .write_to_buffer(&mut buf[11..12])?;
    payload.ref_time.year.write_to_buffer(&mut buf[12..14])?;
    payload.ref_time.month.write_to_buffer(&mut buf[14..15])?;
    payload.ref_time.day.write_to_buffer(&mut buf[15..16])?;
    payload.ref_time.hour.write_to_buffer(&mut buf[16..17])?;
    payload.ref_time.minute.write_to_buffer(&mut buf[17..18])?;
    payload.ref_time.second.write_to_buffer(&mut buf[18..19])?;
    payload.prod_status.write_to_buffer(&mut buf[19..20])?;
    payload.data_type.write_to_buffer(&mut buf[20..21])?;
    Ok(())
}

pub fn write_section8(buf: &mut [u8]) -> Result<(), &'static str> {
    const SIGNATURE: [u8; 4] = [0x37, 0x37, 0x37, 0x37];
    if buf.len() < SIGNATURE.num_bytes_required() {
        return Err("destination buffer is too small");
    }
    SIGNATURE.write_to_buffer(buf)
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
}
