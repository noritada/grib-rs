use std::iter::Peekable;

use crate::decoder::DecodeError;

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
    pub(crate) fn new(bitmap: B, values: I, len: usize) -> Self {
        Self {
            bitmap: bitmap.peekable(),
            values,
            len,
            offset: 0,
        }
    }
}

impl<'b, B, I> Iterator for BitmapDecodeIterator<B, I>
where
    B: Iterator<Item = &'b u8>,
    I: Iterator<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;

        let offset = self.offset;
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
        let size = self.len;
        (size, Some(size))
    }
}

const MASK: u8 = 0b10000000;

fn has_zero_at_offset(byte: &u8, offset: &usize) -> bool {
    let masked = byte & (MASK >> offset);
    masked == 0
}

pub(crate) fn dummy_bitmap_for_nonnullable_data(num_points: usize) -> Vec<u8> {
    let size = super::helpers::num_octets(num_points);
    vec![0b11111111u8; size]
}

pub(crate) fn check_consistency(
    num_points_total: usize,
    num_points_encoded: usize,
    bitmap: &[u8],
) -> Result<(), DecodeError> {
    let num_main_octets = num_points_total / 8;
    let num_remaining_bits = num_points_total % 8;

    let num_octets = if num_remaining_bits == 0 {
        num_main_octets
    } else {
        num_main_octets + 1
    };
    if bitmap.len() < num_octets {
        return Err(DecodeError::LengthMismatch);
    }

    let mut count = bitmap[0..num_main_octets]
        .iter()
        .map(|octet| octet.count_ones())
        .sum::<u32>();
    if num_remaining_bits != 0 {
        count += (bitmap[num_main_octets] >> (8 - num_remaining_bits)).count_ones();
    }
    if count as usize == num_points_encoded {
        Ok(())
    } else {
        Err(DecodeError::UnclassifiedError(
            "inconsistent bitmap".to_owned(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! test_bitmap_iterator {
        ($(($name:ident, $bitmap:expr, $values:expr, $len:expr, $expected:expr,),)*) => ($(
            #[test]
            fn $name() {
                let iter = BitmapDecodeIterator::new($bitmap.iter(), $values.into_iter(), $len);
                let actual = iter.collect::<Vec<_>>();
                let expected = $expected;

                assert_eq!(actual.len(), expected.len());
                actual
                    .iter()
                    .zip(expected.iter())
                    .all(|(a, b)| (a.is_nan() && b.is_nan()) || (a == b));
            }
        )*);
    }

    test_bitmap_iterator! {
        (
            bitmap_iterator_using_bitmap_without_padding,
            [0b01001100u8, 0b01110000, 0b11110000],
            (0..10).map(|n| n as f32).collect::<Vec<_>>(),
            24,
            [
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
            ],
        ),
        (
            bitmap_iterator_using_bitmap_with_padding,
            [0b01001100u8, 0b01110000],
            (0..6).map(|n| n as f32).collect::<Vec<_>>(),
            12,
            [
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
            ],
        ),
    }

    #[test]
    fn bitmap_iterator_size_hint() {
        let bitmap = [0b01001100u8, 0b01110000, 0b11110000];
        let values = (0..10).map(|n| n as f32).collect::<Vec<_>>();
        let values = values.into_iter();

        let mut iter = BitmapDecodeIterator::new(bitmap.iter(), values, 24);

        assert_eq!(iter.size_hint(), (24, Some(24)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (23, Some(23)));
    }

    macro_rules! test_bitmap_consistency {
        ($((
            $name:ident,
            $num_points_total:expr,
            $num_points_encoded:expr,
            $bitmap:expr,
            $expected:expr,
        ),)*) => ($(
            #[test]
            fn $name() {
                let actual = check_consistency($num_points_total, $num_points_encoded, &$bitmap);
                assert_eq!(actual, $expected);
            }
        )*);
    }

    test_bitmap_consistency! {
        (
            bitmap_is_too_short,
            25,
            10,
            [0b01001100u8, 0b01110000, 0b11110000],
            Err(DecodeError::LengthMismatch),
        ),
        (
            num_points_is_divisible_by_8_and_bitmap_is_consistent,
            24,
            10,
            [0b01001100u8, 0b01110000, 0b11110000],
            Ok(()),
        ),
        (
            num_points_is_divisible_by_8_and_bitmap_is_inconsistent,
            24,
            9,
            [0b01001100u8, 0b01110000, 0b11110000],
            Err(DecodeError::UnclassifiedError("inconsistent bitmap".to_owned())),
        ),
        (
            num_points_is_not_divisible_by_8_and_bitmap_is_consistent,
            17,
            7,
            [0b01001100u8, 0b01110000, 0b11110000],
            Ok(()),
        ),
        (
            num_points_is_not_divisible_by_8_and_bitmap_is_inconsistent,
            17,
            6,
            [0b01001100u8, 0b01110000, 0b11110000],
            Err(DecodeError::UnclassifiedError("inconsistent bitmap".to_owned())),
        ),
    }
}
