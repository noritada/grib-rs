use num::ToPrimitive;
use std::cell::RefMut;
use std::convert::TryInto;

use crate::context::SectionInfo;
use crate::decoders::bitmap::BitmapDecodeIterator;
use crate::decoders::common::*;
use crate::error::*;
use crate::reader::Grib2Read;
use crate::utils::{GribInt, NBitwiseIterator};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimplePackingDecodeError {
    NotSupported,
    OriginalFieldValueTypeNotSupported,
    LengthMismatch,
}

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

pub(crate) struct SimplePackingDecoder {}

impl<R: Grib2Read> Grib2DataDecode<R> for SimplePackingDecoder {
    fn decode(
        sect3_num_points: usize,
        sect5: &SectionInfo,
        bitmap: Vec<u8>,
        sect7: &SectionInfo,
        mut reader: RefMut<R>,
    ) -> Result<Box<[f32]>, GribError> {
        let sect5_data = reader.read_sect_body_bytes(sect5)?;
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

        let sect7_data = reader.read_sect_body_bytes(sect7)?;

        // Based on the implementation of wgrib2, if nbits equals 0, return a constant field where
        // the data value at each grid point is the reference value.
        if nbit == 0 {
            let decoded = vec![ref_val; sect3_num_points];
            return Ok(decoded.into_boxed_slice());
        }

        let iter = NBitwiseIterator::new(&sect7_data, usize::from(nbit));
        let decoder = SimplePackingDecodeIterator::new(iter, ref_val, exp, dig);
        // Taking first `num_points` is needed.  Since the bitmap is represented as a
        // sequence of bytes, for example, if there are 9 grid points, the
        // number of iterations will probably be 16, which is greater than the
        // original number of grid points.
        let decoder = BitmapDecodeIterator::new(bitmap.iter(), decoder).take(sect3_num_points);
        let decoded = decoder.collect::<Vec<_>>();
        if decoded.len() != sect3_num_points {
            return Err(GribError::DecodeError(
                DecodeError::SimplePackingDecodeError(SimplePackingDecodeError::LengthMismatch),
            ));
        }
        Ok(decoded.into_boxed_slice())
    }
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
                let diff = (encoded * 2_f32.powi(self.exp)) as f32;
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
    use crate::decoders::bitmap::create_bitmap_for_nonnullable_data;

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
        let index = 0;
        let (sect3, sect5, _sect6, sect7) = grib
            .submessages
            .get(index)
            .and_then(|submsg| {
                Some((
                    grib.sections.get(submsg.section3)?,
                    grib.sections.get(submsg.section5)?,
                    grib.sections.get(submsg.section6)?,
                    grib.sections.get(submsg.section7)?,
                ))
            })
            .ok_or(GribError::InternalDataError)
            .unwrap();

        let reader = grib.reader.borrow_mut();

        // FIXME: Bitmap creation process is hardcoded, assuming that the bitmap
        // indicator is 0xff.
        let num_points = if let Some(SectionBody::Section3(sect3_body)) = &sect3.body {
            sect3_body.num_points as usize
        } else {
            0
        };
        let bitmap = create_bitmap_for_nonnullable_data(num_points);

        let actual =
            SimplePackingDecoder::decode(num_points, sect5, bitmap, sect7, reader).unwrap();
        let expected = vec![0f32; 0x002d0000].into_boxed_slice();
        assert_eq!(actual, expected);
    }
}
