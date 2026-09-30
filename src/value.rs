/// Missing values defined in the GRIB2 regulation.
///
/// Regulation 92.1.4 states as follows:
///
/// > All bits set to "1" for any value indicates that value is missing. This
/// > rule shall not apply to packed data.
///
/// # Examples
///
/// ```
/// use grib::{MissingValue, TryFromSlice};
///
/// let missing = i32::missing();
/// let read_from_slice =
///     i32::try_from_slice(&[0xff, 0xff, 0xff, 0xff].as_slice(), &mut 0).unwrap();
/// assert_eq!(missing, read_from_slice);
/// assert!(missing.is_missing());
/// ```
pub trait MissingValue {
    /// Returns missing value.
    fn missing() -> Self;

    /// Checks if the value is regarded as a missing value.
    fn is_missing(&self) -> bool;
}

macro_rules! add_missing_value_impl_for_unsigned_integer_types {
    ($($ty:ty,)*) => ($(
        impl MissingValue for $ty {
            fn missing() -> Self {
                Self::MAX
            }

            fn is_missing(&self) -> bool {
                *self == Self::MAX
            }
        }
    )*);
}

add_missing_value_impl_for_unsigned_integer_types![u8, u16, u32, u64,];

macro_rules! add_missing_value_impl_for_signed_integer_types {
    ($($ty:ty,)*) => ($(
        impl MissingValue for $ty {
            fn missing() -> Self {
                Self::MIN + 1
            }

            fn is_missing(&self) -> bool {
                *self == Self::MIN + 1
            }
        }
    )*);
}

add_missing_value_impl_for_signed_integer_types![i8, i16, i32, i64,];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TryFromSlice as _;

    macro_rules! test_missing_values {
        ($(($name:ident, $ty:ty),)*) => ($(
            #[test]
            fn $name() -> Result<(), Box<dyn std::error::Error>> {
                let expected = [0xff_u8; (<$ty>::BITS / 8) as usize];
                let expected = <$ty>::try_from_slice(expected.as_slice(), &mut 0)?;
                let actual = <$ty>::missing();
                assert_eq!(actual, expected);
                assert!(actual.is_missing());
                Ok(())
            }
        )*);
    }

    test_missing_values! {
        (missing_value_for_u8, u8),
        (missing_value_for_u16, u16),
        (missing_value_for_u32, u32),
        (missing_value_for_u64, u64),
        (missing_value_for_i8, i8),
        (missing_value_for_i16, i16),
        (missing_value_for_i32, i32),
        (missing_value_for_i64, i64),
    }
}
