use crate::encoder::to_grib_signed::ToGribSigned as _;

pub(crate) trait WriteToBuffer {
    fn write_to_buffer(&self, buf: &mut Vec<u8>) -> Result<(), &'static str>;
    fn num_bytes_required(&self) -> usize;
}

macro_rules! add_impl_for_unsigned_integer_types {
    ($($ty:ty,)*) => ($(
        impl WriteToBuffer for $ty {
            fn write_to_buffer(&self, buf: &mut Vec<u8>) -> Result<(), &'static str> {
                if buf.len() < self.num_bytes_required() {
                    return Err("destination buffer is too small");
                }

                let bytes = self.to_be_bytes();
                for i in 0..self.num_bytes_required() {
                    buf[i] = bytes[i];
                }
                Ok(())
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
            fn write_to_buffer(&self, buf: &mut Vec<u8>) -> Result<(), &'static str> {
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
    fn write_to_buffer(&self, buf: &mut Vec<u8>) -> Result<(), &'static str> {
        self.to_bits().write_to_buffer(buf)
    }

    fn num_bytes_required(&self) -> usize {
        self.to_bits().num_bytes_required()
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
}
