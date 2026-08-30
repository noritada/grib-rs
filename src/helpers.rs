use crate::TryFromSlice as _;

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
    let mut pos = 0;
    match len {
        1 => i32::from(i8::try_from_slice(bytes, &mut pos).unwrap()),
        2 => i32::from(i16::try_from_slice(bytes, &mut pos).unwrap()),
        3 => {
            let first = u8::try_from_slice(bytes, &mut pos).unwrap();
            let positive = first.leading_zeros() != 0;
            let rest = i32::from(u16::try_from_slice(bytes, &mut pos).unwrap());
            let abs = i32::from(first << 1 >> 1) * 0x10000 + rest;
            if positive { abs } else { -abs }
        }
        4 => i32::try_from_slice(bytes, &mut pos).unwrap(),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
