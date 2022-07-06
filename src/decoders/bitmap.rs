use std::iter::Peekable;

pub(crate) struct BitmapDecodeIterator<B: Iterator, I> {
    bitmap: Peekable<B>,
    values: I,
    offset: usize,
}

impl<'b, B, I> BitmapDecodeIterator<B, I>
where
    B: Iterator<Item = &'b u8>,
{
    pub(crate) fn new(bitmap: B, values: I) -> Self {
        Self {
            bitmap: bitmap.peekable(),
            values,
            offset: 0,
        }
    }
}

impl<'b, B, I, N> Iterator for BitmapDecodeIterator<B, I>
where
    B: Iterator<Item = &'b u8>,
    I: Iterator<Item = N>,
    Option<f32>: From<Option<N>>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
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
            self.values.next().into()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (b_min, b_max) = self.bitmap.size_hint();
        let min = b_min * 8 - 7;
        if let Some(max) = b_max {
            (min, Some(max * 8))
        } else {
            (min, None)
        }
    }
}

const MASK: u8 = 0b10000000;

fn has_zero_at_offset(byte: &u8, offset: &usize) -> bool {
    let masked = byte & (MASK >> offset);
    masked == 0
}
