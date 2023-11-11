pub(crate) enum BitStream<T> {
    ZeroSized(FixedValueIterator<u32>),
    NonZeroSized(NBitwiseIterator<T>),
}

impl<T> BitStream<T> {
    pub(crate) fn new(data: T, unit_size: usize, length: usize) -> Self {
        if unit_size == 0 {
            let iter = FixedValueIterator::new(0, length);
            Self::ZeroSized(iter)
        } else {
            let iter = NBitwiseIterator::new(data, unit_size);
            Self::NonZeroSized(iter)
        }
    }
}

impl<T> Iterator for BitStream<T>
where
    T: AsRef<[u8]>,
{
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::ZeroSized(z) => z.next(),
            Self::NonZeroSized(n) => n.next(),
        }
    }
}

pub(crate) struct FixedValueIterator<T> {
    val: T,
    length: usize,
    pos: usize,
}

impl<T> FixedValueIterator<T> {
    pub(crate) fn new(val: T, length: usize) -> Self {
        Self {
            val,
            length,
            pos: 0,
        }
    }
}

impl<T> Iterator for FixedValueIterator<T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let val = if self.pos < self.length {
            Some(self.val)
        } else {
            None
        };
        self.pos += 1;
        val
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.length - self.pos;
        (size, Some(size))
    }
}

#[derive(Clone)]
pub(crate) struct NBitwiseIterator<T> {
    pub(crate) data: T,
    pub(crate) size: usize,
    pub(crate) pos: usize,
    pub(crate) offset: usize,
}

impl<T> NBitwiseIterator<T> {
    pub(crate) fn new(data: T, size: usize) -> Self {
        Self {
            data,
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

impl<T> Iterator for NBitwiseIterator<T>
where
    T: AsRef<[u8]>,
{
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let new_offset = self.offset + self.size;
        let (new_pos, new_offset) = (self.pos + new_offset / 8, new_offset % 8);
        let slice = self.data.as_ref();

        if self.pos >= slice.len()
            || new_pos > slice.len()
            || (new_pos == slice.len() && new_offset > 0)
        {
            return None;
        }

        let val = slice[self.pos] << self.offset >> self.offset;
        let mut val: u32 = u32::from(val);
        if new_pos == self.pos {
            val >>= 8 - new_offset;
        } else {
            let mut pos = self.pos + 1;
            while pos < new_pos {
                val = (val << 8) | u32::from(slice[pos]);
                pos += 1;
            }
            if new_offset > 0 {
                let shift = 8 - new_offset;
                let last_val = u32::from(slice[pos]) >> shift;
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

    #[test]
    fn nbitwise_iterator_u2() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(&slice, 2);
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

        let mut iter = NBitwiseIterator::new(&slice, 5);
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

        let mut iter = NBitwiseIterator::new(&slice, 9);
        assert_eq!(iter.next(), Some(0b000000001));
        assert_eq!(iter.next(), Some(0b111111111));
        assert_eq!(iter.next(), Some(0b111111000));
        assert_eq!(iter.next(), Some(0b000000000));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn nbitwise_iterator_u13() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(&slice, 13);
        assert_eq!(iter.next(), Some(0b0000000011111));
        assert_eq!(iter.next(), Some(0b1111111111100));
        assert_eq!(iter.next(), Some(0b0000000000000));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn nbitwise_iterator_with_offset() {
        let slice: [u8; 5] = [0, 255, 255, 0, 0];

        let mut iter = NBitwiseIterator::new(&slice, 2).with_offset(7);
        assert_eq!(iter.next(), Some(0b01));
    }

    #[test]
    fn nbitwise_iterator_empty() {
        let slice: [u8; 0] = [];

        let mut iter = NBitwiseIterator::new(&slice, 0);
        assert_eq!(iter.next(), None);
    }
}
