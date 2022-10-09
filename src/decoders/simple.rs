use num::ToPrimitive;
use std::convert::TryInto;

use crate::decoders::common::*;
use crate::error::*;
use crate::utils::{read_as, GribInt, NBitwiseIterator};

pub(crate) enum SimplePackingDecodeIteratorWrapper<I> {
    FixedValue(std::vec::IntoIter<f32>),
    SimplePacking(SimplePackingDecodeIterator<I>),
}

impl<I, N> Iterator for SimplePackingDecodeIteratorWrapper<I>
where
    I: Iterator<Item = N>,
    N: ToPrimitive,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FixedValue(inner) => inner.next(),
            Self::SimplePacking(inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::FixedValue(inner) => inner.size_hint(),
            Self::SimplePacking(inner) => inner.size_hint(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimplePackingDecodeError {
    NotSupported,
    OriginalFieldValueTypeNotSupported,
    LengthMismatch,
}

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIteratorWrapper<impl Iterator<Item = u32> + '_>, GribError> {
    let sect5_data = &target.sect5_payload;
    let ref_val = read_as!(f32, sect5_data, 6);
    let exp = read_as!(u16, sect5_data, 10).as_grib_int();
    let dig = read_as!(u16, sect5_data, 12).as_grib_int();
    let nbit = read_as!(u8, sect5_data, 14);
    let value_type = read_as!(u8, sect5_data, 15);

    if value_type != 0 {
        return Err(GribError::DecodeError(
            DecodeError::SimplePackingDecodeError(
                SimplePackingDecodeError::OriginalFieldValueTypeNotSupported,
            ),
        ));
    }

    let decoder = if nbit == 0 {
        // Based on the implementation of wgrib2, if nbits equals 0, return a constant
        // field where the data value at each grid point is the reference value.
        let decoded = vec![ref_val; target.num_points_encoded];
        SimplePackingDecodeIteratorWrapper::FixedValue(decoded.into_iter())
    } else {
        let iter = NBitwiseIterator::new(&target.sect7_payload, usize::from(nbit));
        let iter = SimplePackingDecodeIterator::new(iter, ref_val, exp, dig);
        SimplePackingDecodeIteratorWrapper::SimplePacking(iter)
    };
    Ok(decoder)
}

pub(crate) struct SimplePackingDecodeIterator<I> {
    iter: I,
    ref_val: f32,
    exp: i32,
    dig: i32,
}

impl<I> SimplePackingDecodeIterator<I> {
    pub(crate) fn new(iter: I, ref_val: f32, exp: i16, dig: i16) -> Self {
        Self {
            iter,
            ref_val,
            exp: exp.into(),
            dig: dig.into(),
        }
    }
}

impl<I: Iterator<Item = N>, N: ToPrimitive> Iterator for SimplePackingDecodeIterator<I> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        match self.iter.next() {
            Some(encoded) => {
                let encoded = encoded.to_f32().unwrap();
                let diff = encoded * 2_f32.powi(self.exp);
                let dig_factor = 10_f32.powi(-self.dig);
                let value: f32 = (self.ref_val + diff) * dig_factor;
                Some(value)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::{BufReader, Cursor, Read};

    use crate::context::from_reader;

    #[test]
    fn decode_simple_packing() {
        let ref_val_bytes = vec![0x35, 0x3e, 0x6b, 0xf6];
        let exp = 0x801a;
        let dig = 0x0000;
        let input: Vec<u8> = vec![0x00, 0x06, 0x00, 0x0d];
        let expected: Vec<f32> = vec![7.987_831_6e-7, 9.030_913e-7];

        let ref_val = f32::from_be_bytes(ref_val_bytes[..].try_into().unwrap());
        let iter = NBitwiseIterator::new(&input, 16);
        let actual =
            SimplePackingDecodeIterator::new(iter, ref_val, exp.as_grib_int(), dig.as_grib_int())
                .collect::<Vec<_>>();

        assert_eq!(actual.len(), expected.len());
        let mut i = 0;
        while i < actual.len() {
            assert!(actual[i] < expected[i] + 0.00000001);
            assert!(actual[i] > expected[i] - 0.00000001);
            i += 1;
        }
    }

    #[test]
    fn decode_simple_packing_when_nbit_is_zero() {
        let f = File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )
        .unwrap();
        let mut f = BufReader::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        let f = Cursor::new(buf);

        let grib = from_reader(f).unwrap();
        let message_index = (0, 0);
        let (_, submessage) = grib
            .iter()
            .find(|(index, _)| *index == message_index)
            .unwrap();
        let decoder = Grib2SubmessageDecoder::from(submessage).unwrap();
        // Runs `SimplePackingDecoder::decode()` internally.
        let actual = decoder.dispatch().unwrap().collect::<Vec<_>>();
        let expected = vec![0f32; 0x002d0000];
        assert_eq!(actual, expected);
    }
}
