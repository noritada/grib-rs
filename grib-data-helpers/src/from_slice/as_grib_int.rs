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

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

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
}
