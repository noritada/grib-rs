use num::ToPrimitive;

use crate::{
    decoder::{
        param::SimplePackingParam,
        stream::{FixedValueIterator, NBitwiseIterator},
        Grib2SubmessageDecoder,
    },
    error::*,
};

pub(crate) enum SimplePackingDecodeIteratorWrapper<I> {
    // Based on the implementation of wgrib2, if nbits equals 0, return a constant
    // field where the data value at each grid point is the reference value.
    FixedValue(FixedValueIterator<f32>),
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

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIteratorWrapper<impl Iterator<Item = u32> + '_>, GribError> {
    let sect5_data = &target.sect5_bytes;
    let param = SimplePackingParam::from_buf(&sect5_data[11..21])?;

    let decoder = if param.nbit == 0 {
        SimplePackingDecodeIteratorWrapper::FixedValue(FixedValueIterator::new(
            param.zero_bit_reference_value(),
            target.num_points_encoded(),
        ))
    } else {
        let iter = NBitwiseIterator::new(target.sect7_payload(), usize::from(param.nbit));
        let iter = SimplePackingDecodeIterator::new(iter, &param);
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
    pub(crate) fn new(iter: I, param: &SimplePackingParam) -> Self {
        Self {
            iter,
            ref_val: param.ref_val,
            exp: param.exp.into(),
            dig: param.dig.into(),
        }
    }
}

impl<I: Iterator<Item = N>, N: ToPrimitive> Iterator for SimplePackingDecodeIterator<I> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
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
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::*;

    #[test]
    fn decode_simple_packing() {
        let buf = vec![0x35, 0x3e, 0x6b, 0xf6, 0x80, 0x1a, 0x00, 0x00, 0x10, 0x00];
        let param = SimplePackingParam::from_buf(&buf).unwrap();
        let input: Vec<u8> = vec![0x00, 0x06, 0x00, 0x0d];
        let expected: Vec<f32> = vec![7.987_831_6e-7, 9.030_913e-7];

        let iter = NBitwiseIterator::new(&input, usize::from(param.nbit));
        let actual = SimplePackingDecodeIterator::new(iter, &param).collect::<Vec<_>>();

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

        let decoder = Grib2SubmessageDecoder::new(
            2949120,
            buf[0x0000009d..0x000000b2].to_vec(),
            buf[0x000000b2..0x000000b8].to_vec(),
            buf[0x000000b8..0x000000bd].to_vec(),
        )
        .unwrap();
        // Runs `SimplePackingDecoder::decode()` internally.
        let actual = decoder.dispatch().unwrap().collect::<Vec<_>>();
        let expected = vec![0f32; 0x002d0000];
        assert_eq!(actual, expected);
    }
}
