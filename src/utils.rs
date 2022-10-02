pub(crate) trait GribInt<I> {
    fn as_grib_int(&self) -> I;
}

macro_rules! add_impl_for_ints {
    ($(($ty_src:ty, $ty_dst:ty),)*) => ($(
        impl GribInt<$ty_dst> for $ty_src {
            fn as_grib_int(&self) -> $ty_dst {
                if self.leading_zeros() == 0 {
                    let abs = (self << 1 >> 1) as $ty_dst;
                    -abs
                } else {
                    *self as $ty_dst
                }
            }
        }
    )*);
}

add_impl_for_ints! {
    (u8, i8),
    (u16, i16),
    (u32, i32),
    (u64, i64),
}

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}
pub(crate) use read_as;

#[derive(Clone)]
pub(crate) struct NBitwiseIterator {
    slice: Vec<u8>,
    size: usize,
    pos: usize,
    offset: usize,
}

impl NBitwiseIterator {
    pub(crate) fn new(slice: Vec<u8>, size: usize) -> Self {
        Self {
            slice,
            size,
            pos: 0,
            offset: 0,
        }
    }

    pub(crate) fn with_offset(self, offset_bits: usize) -> Self {
        Self {
            offset: offset_bits,
            ..self
        }
    }
}

impl Iterator for NBitwiseIterator {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        let new_offset = self.offset + self.size;
        let (new_pos, new_offset) = (self.pos + new_offset / 8, new_offset % 8);

        if self.pos >= self.slice.len()
            || new_pos > self.slice.len()
            || (new_pos == self.slice.len() && new_offset > 0)
        {
            return None;
        }

        let val = self.slice[self.pos] << self.offset >> self.offset;
        let mut val: u32 = u32::from(val);
        if new_pos == self.pos {
            val >>= 8 - new_offset;
        } else {
            let mut pos = self.pos + 1;
            while pos < new_pos {
                val = (val << 8) | u32::from(self.slice[pos]);
                pos += 1;
            }
            if new_offset > 0 {
                let shift = 8 - new_offset;
                let last_val = u32::from(self.slice[pos]) >> shift;
                val = (val << new_offset) | last_val;
            }
        }

        self.pos = new_pos;
        self.offset = new_offset;
        Some(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::convert::TryInto;

    #[test]
    fn into_grib_i8() {
        let input: Vec<u8> = vec![0b01000000, 0b00000001, 0b10000001, 0b11000000];
        let output: Vec<i8> = vec![64, 1, -1, -64];

        let mut actual = Vec::new();
        let mut pos = 0;
        while pos < input.len() {
            let val = u8::from_be_bytes(input[pos..pos + 1].try_into().unwrap());
            pos += 1;
            let val = val.as_grib_int();
            actual.push(val);
        }

        assert_eq!(actual, output);
    }

    #[test]
    fn into_grib_i16() {
        let input: Vec<u8> = vec![
            0b00000000, 0b01000000, 0b00000000, 0b00000001, 0b10000000, 0b00000001, 0b10000000,
            0b01000000,
        ];
        let output: Vec<i16> = vec![64, 1, -1, -64];

        let mut actual = Vec::new();
        let mut pos = 0;
        while pos < input.len() {
            let val = u16::from_be_bytes(input[pos..pos + 2].try_into().unwrap());
            pos += 2;
            let val = val.as_grib_int();
            actual.push(val);
        }

        assert_eq!(actual, output);
    }

    #[test]
    fn nbitwise_iterator_u2() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(slice.to_vec(), 2);
        assert_eq!(iter.next(), Some(0b00));
        assert_eq!(iter.next(), Some(0b00));
        assert_eq!(iter.next(), Some(0b00));
        assert_eq!(iter.next(), Some(0b00));
        assert_eq!(iter.next(), Some(0b11));
        assert_eq!(iter.next(), Some(0b11));
    }

    #[test]
    fn nbitwise_iterator_u5() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(slice.to_vec(), 5);
        assert_eq!(iter.next(), Some(0b00000));
        assert_eq!(iter.next(), Some(0b00011));
        assert_eq!(iter.next(), Some(0b11111));
        assert_eq!(iter.next(), Some(0b11111));
        assert_eq!(iter.next(), Some(0b11110));
        assert_eq!(iter.next(), Some(0b00000));
        assert_eq!(iter.next(), Some(0b00000));
        assert_eq!(iter.next(), Some(0b00000));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn nbitwise_iterator_u9() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(slice.to_vec(), 9);
        assert_eq!(iter.next(), Some(0b000000001));
        assert_eq!(iter.next(), Some(0b111111111));
        assert_eq!(iter.next(), Some(0b111111000));
        assert_eq!(iter.next(), Some(0b000000000));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn nbitwise_iterator_u13() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(slice.to_vec(), 13);
        assert_eq!(iter.next(), Some(0b0000000011111));
        assert_eq!(iter.next(), Some(0b1111111111100));
        assert_eq!(iter.next(), Some(0b0000000000000));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn nbitwise_iterator_with_offset() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(slice.to_vec(), 2).with_offset(7);
        assert_eq!(iter.next(), Some(0b01));
    }

    #[test]
    fn nbitwise_iterator_empty() {
        let slice: [u8; 0] = [];

        let mut iter = NBitwiseIterator::new(slice.to_vec(), 0);
        assert_eq!(iter.next(), None);
    }
}
