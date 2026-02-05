use crate::encoder::to_grib_signed::ToGribSigned as _;

pub trait WriteToBuffer {
    fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
    fn num_bytes_required(&self) -> usize;
}

macro_rules! add_impl_for_unsigned_integer_types {
    ($($ty:ty,)*) => ($(
        impl WriteToBuffer for $ty {
            fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                let len = self.num_bytes_required();
                if buf.len() < len {
                    return Err("destination buffer is too small");
                }

                let bytes = self.to_be_bytes();
                for i in 0..self.num_bytes_required() {
                    buf[i] = bytes[i];
                }
                Ok(len)
            }

            fn num_bytes_required(&self) -> usize {
                (Self::BITS / 8) as usize
            }
        }
    )*);
}

add_impl_for_unsigned_integer_types![u8, u16, u32, u64,];

macro_rules! add_impl_for_signed_integer_types {
    ($($ty:ty,)*) => ($(
        impl WriteToBuffer for $ty {
            fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                self.to_grib_signed().write_to_buffer(buf)
            }

            fn num_bytes_required(&self) -> usize {
                self.to_grib_signed().num_bytes_required()
            }
        }
    )*);
}

add_impl_for_signed_integer_types![i8, i16, i32,];

impl WriteToBuffer for f32 {
    fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        self.to_bits().write_to_buffer(buf)
    }

    fn num_bytes_required(&self) -> usize {
        self.to_bits().num_bytes_required()
    }
}

impl<const N: usize> WriteToBuffer for [u8; N] {
    fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if buf.len() < N {
            return Err("destination buffer is too small");
        }

        buf.copy_from_slice(self);
        Ok(N)
    }

    fn num_bytes_required(&self) -> usize {
        N
    }
}

#[derive(Clone)]
pub(crate) struct NBitwise<B> {
    data: B,
    num_bits: usize,
}

impl<B> NBitwise<B> {
    pub(crate) fn new(data: B, num_bits: usize) -> Self {
        Self { data, num_bits }
    }
}

impl<B: AsRef<[u32]>> WriteToBuffer for NBitwise<B> {
    fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        if self.num_bits == 0 {
            return Err("invalid `num_bits` value");
        }
        let len = self.num_bytes_required();
        if buf.len() < len {
            return Err("destination buffer is too small");
        }

        let (mut current_pos, mut current_offset) = (0, 0);

        const BYTE_MASK: u32 = 0xff;
        if self.num_bits % 8 == 0 {
            for item in self.data.as_ref() {
                let mut num_bits = self.num_bits;
                while num_bits > 0 {
                    num_bits -= 8;
                    buf[current_pos] = ((item >> num_bits) & BYTE_MASK) as u8;
                    current_pos += 1;
                }
            }
        } else {
            for item in self.data.as_ref() {
                let mut num_bits = self.num_bits;
                while num_bits > 0 {
                    let window_size = (8 - current_offset).min(num_bits);
                    let pad_size = 8 - 8.min(current_offset + num_bits);
                    num_bits -= window_size;
                    let value =
                        (((item >> num_bits) & (BYTE_MASK >> current_offset)) as u8) << pad_size;
                    if current_offset == 0 {
                        buf[current_pos] = value;
                    } else {
                        buf[current_pos] |= value;
                    }
                    current_offset += window_size;
                    if current_offset >= 8 {
                        current_pos += 1;
                        current_offset -= 8;
                    }
                }
            }
        }

        Ok(len)
    }

    fn num_bytes_required(&self) -> usize {
        (self.num_bits * self.data.as_ref().len()).div_ceil(8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_writing_to_buffer {
        ($(($name:ident, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let mut buf = vec![0; 4];
                let result = $input.write_to_buffer(&mut buf);
                assert!(result.is_ok());
                assert_eq!(buf, $expected);
            }
        )*);
    }

    test_writing_to_buffer! {
        (writing_u8_to_buffer, 1_u8, vec![1, 0, 0, 0]),
        (writing_u16_to_buffer, 1_u16, vec![0, 1, 0, 0]),
        (writing_u32_to_buffer, 1_u32, vec![0, 0, 0, 1]),
    }

    macro_rules! test_nbitwise {
        ($(($name:ident, $input:expr, $num_bits:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let src = NBitwise::new($input, $num_bits);
                let mut dirty_buf = vec![1; src.num_bytes_required()];
                let result = src.write_to_buffer(&mut dirty_buf);
                assert!(result.is_ok());
                assert_eq!(dirty_buf, $expected);
            }
        )*);
    }

    test_nbitwise! {
        (
            nbitwise_for_0_modulo_8,
            (0_u32..11).collect::<Vec<_>>(),
            16,
            vec![
                0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x03, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06,
                0x00, 0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x0a,
            ]
        ),
        (
            nbitwise_for_4_modulo_8,
            (0_u32..11).collect::<Vec<_>>(),
            4,
            vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xa0]
        ),
        (
            nbitwise_for_5_modulo_8,
            (0_u32..11).collect::<Vec<_>>(),
            5,
            vec![
                0b00000_000, 0b01_00010_0, 0b0011_0010, 0b0_00101_00, 0b110_00111, 0b01000_010,
                0b01_01010_0
            ]
        ),
        (
            nbitwise_for_3_modulo_8_larger_than_8,
            (0_u32..11).collect::<Vec<_>>(),
            11,
            vec![
                0b00000000, 0b000_00000, 0b000001_00, 0b00000001, 0b0_0000000, 0b0011_0000,
                0b0000100_0, 0b00000001, 0b01_000000, 0b00110_000, 0b00000111, 0b00000001,
                0b000_00000, 0b001001_00, 0b00000101, 0b0_0000000
            ]
        ),
    }
}
