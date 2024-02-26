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

pub(crate) fn grib_int_from_bytes(bytes: &[u8]) -> i32 {
    let len = bytes.len();
    // Although there is logic that can be used to generalize, not so many patterns
    // exist that generalization is necessary.
    match len {
        1 => i32::from(read_as!(u8, bytes, 0).as_grib_int()),
        2 => i32::from(read_as!(u16, bytes, 0).as_grib_int()),
        3 => {
            let first = read_as!(u8, bytes, 0);
            let positive = first.leading_zeros() != 0;
            let rest = i32::from(read_as!(u16, bytes, 1));
            let abs = i32::from(first << 1 >> 1) * 0x10000 + rest;
            if positive {
                abs
            } else {
                -abs
            }
        }
        4 => read_as!(u32, bytes, 0).as_grib_int(),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
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

    macro_rules! test_conversion_from_bytes_to_grib_int {
        ($(($name:ident, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let bytes = $input;
                let actual = grib_int_from_bytes(&bytes);
                let expected = $expected;
                assert_eq!(actual, expected)
            }
        )*);
    }

    test_conversion_from_bytes_to_grib_int! {
        (
            conversion_from_bytes_to_grib_int_for_1_byte_positive,
            vec![0b01010101],
            0b01010101
        ),
        (
            conversion_from_bytes_to_grib_int_for_1_byte_negative,
            vec![0b11010101],
            -0b01010101
        ),
        (
            conversion_from_bytes_to_grib_int_for_2_bytes_positive,
            vec![0b01010101, 0b10101010],
            0b0101_0101_1010_1010
        ),
        (
            conversion_from_bytes_to_grib_int_for_2_bytes_negative,
            vec![0b11010101, 0b10101010],
            -0b0101_0101_1010_1010
        ),
        (
            conversion_from_bytes_to_grib_int_for_3_bytes_positive,
            vec![0b01010101, 0b10101010, 0b10101010],
            0b0101_0101_1010_1010_1010_1010
        ),
        (
            conversion_from_bytes_to_grib_int_for_3_bytes_negative,
            vec![0b11010101, 0b10101010, 0b10101010],
            -0b0101_0101_1010_1010_1010_1010
        ),
        (
            conversion_from_bytes_to_grib_int_for_3_bytes_negative_starting_from_0x80,
            vec![0b10000000, 0b10101010, 0b10101010],
            -0b0000_0000_1010_1010_1010_1010
        ),
        (
            conversion_from_bytes_to_grib_int_for_4_bytes_positive,
            vec![0b01010101, 0b10101010, 0b10101010, 0b10101010],
            0b0101_0101_1010_1010_1010_1010_1010_1010
        ),
        (
            conversion_from_bytes_to_grib_int_for_4_bytes_negative,
            vec![0b11010101, 0b10101010, 0b10101010, 0b10101010],
            -0b0101_0101_1010_1010_1010_1010_1010_1010
        ),
    }
}
