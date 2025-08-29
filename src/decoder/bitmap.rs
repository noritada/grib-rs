use std::iter::Peekable;

use crate::{decoder::DecodeError, error::GribError};

pub(crate) struct BitmapDecodeIterator<B: Iterator, I> {
    bitmap: Peekable<B>,
    values: I,
    len: usize,
    offset: usize,
}

impl<'b, B, I> BitmapDecodeIterator<B, I>
where
    B: Iterator<Item = &'b u8>,
{
    pub(crate) fn new(bitmap: B, values: I, len: usize) -> Result<Self, GribError> {
        let (bitmap_len, _) = bitmap.size_hint();
        if bitmap_len * 8 < len {
            return Err(GribError::DecodeError(DecodeError::LengthMismatch));
        }
        Ok(Self {
            bitmap: bitmap.peekable(),
            values,
            len,
            offset: 0,
        })
    }
}

impl<'b, B, I> Iterator for BitmapDecodeIterator<B, I>
where
    B: Iterator<Item = &'b u8>,
    I: Iterator<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.offset;
        if offset >= self.len {
            return None;
        }
        let byte = if self.offset < 7 {
            self.offset += 1;
            self.bitmap.peek()?
        } else {
            self.offset = 0;
            self.bitmap.next()?
        };

        if has_zero_at_offset(byte, &offset) {
            Some(f32::NAN)
        } else {
            self.values.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.offset;
        (size, Some(size))
    }
}

const MASK: u8 = 0b10000000;

fn has_zero_at_offset(byte: &u8, offset: &usize) -> bool {
    let masked = byte & (MASK >> offset);
    masked == 0
}

pub(crate) fn create_bitmap_for_nonnullable_data(num_points: usize) -> Vec<u8> {
    let size = num_octets_for_bitmap(num_points);
    vec![0b11111111u8; size]
}

pub(crate) fn num_octets_for_bitmap(num_points: usize) -> usize {
    (num_points + 7) / 8
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bitmap_iterator_works() {
        let bitmap = [0b01001100u8, 0b01110000, 0b11110000];
        let values = (0..10).map(|n| n as f32).collect::<Vec<_>>();
        let values = values.into_iter();

        let iter = BitmapDecodeIterator::new(bitmap.iter(), values, 24).unwrap();
        let actual = iter.collect::<Vec<_>>();
        let expected = [
            f32::NAN,
            0.0,
            f32::NAN,
            f32::NAN,
            1.0,
            2.0,
            f32::NAN,
            f32::NAN,
            f32::NAN,
            3.0,
            4.0,
            5.0,
            f32::NAN,
            f32::NAN,
            f32::NAN,
            f32::NAN,
            6.0,
            7.0,
            8.0,
            9.0,
            f32::NAN,
            f32::NAN,
            f32::NAN,
            f32::NAN,
        ];

        actual
            .iter()
            .zip(expected.iter())
            .all(|(a, b)| (a.is_nan() && b.is_nan()) || (a == b));
    }

    #[test]
    fn bitmap_iterator_size_hint() {
        let bitmap = [0b01001100u8, 0b01110000, 0b11110000];
        let values = (0..10).map(|n| n as f32).collect::<Vec<_>>();
        let values = values.into_iter();

        let mut iter = BitmapDecodeIterator::new(bitmap.iter(), values, 24).unwrap();

        assert_eq!(iter.size_hint(), (24, Some(24)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (23, Some(23)));
    }

    #[test]
    fn bitmap_size_calculation() {
        let actual = (0..16)
            .map(|n| num_octets_for_bitmap(n))
            .collect::<Vec<_>>();
        let expected = [0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2];
        assert_eq!(actual, expected);
    }
}
