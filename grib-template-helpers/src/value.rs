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
/// use grib_template_helpers::{MissingValue, TryFromSlice};
///
/// let missing = i32::missing();
/// let read_from_slice =
///     i32::try_from_slice(&[0xff, 0xff, 0xff, 0xff].as_slice(), &mut 0).unwrap();
/// assert_eq!(missing, read_from_slice);
/// assert!(missing.is_missing());
/// ```
pub trait MissingValue {
    /// Returns missing value.
    ///
    /// For `NonStdLenUint`, the result uses the full width of its underlying
    /// integer type. To represent a missing value of a shorter width, construct
    /// it with `NonStdLenUint::new` and all bits set within that width.
    fn missing() -> Self;

    /// Checks if the value is regarded as a missing value.
    fn is_missing(&self) -> bool;
}

macro_rules! add_missing_value_impl_for_unsigned_integer_types {
    ($($ty:ty),* $(,)?) => ($(
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

add_missing_value_impl_for_unsigned_integer_types![u8, u16, u32, u64];

macro_rules! add_missing_value_impl_for_signed_integer_types {
    ($($ty:ty),* $(,)?) => ($(
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

add_missing_value_impl_for_signed_integer_types![i8, i16, i32, i64];

macro_rules! impl_float {
    ($($ty:ty => $bits:ty),* $(,)?) => {$(
        impl MissingValue for $ty {
            fn missing() -> Self { Self::from_bits(<$bits>::MAX) }
            fn is_missing(&self) -> bool { self.to_bits() == <$bits>::MAX }
        }
    )*};
}
impl_float!(f32 => u32, f64 => u64);

macro_rules! impl_non_standard {
    ($($ty:ty),* $(,)?) => {$(
        impl MissingValue for crate::NonStdLenUint<$ty> {
            // Without a supplied length, construct a full-width missing value.
            fn missing() -> Self {
                Self::new(<$ty>::MAX, std::mem::size_of::<$ty>())
            }
            fn is_missing(&self) -> bool {
                let width = std::mem::size_of::<$ty>();
                let len = self.num_octets();
                len > 0 && len <= width
                    && *self.val() == (<$ty>::MAX >> ((width - len) * 8))
            }
        }
    )*};
}
impl_non_standard!(u8, u16, u32, u64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TryFromSlice as _;

    macro_rules! test_missing_values {
        ($(($name:ident, $ty:ty $(,)?)),* $(,)?) => ($(
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

#[cfg(test)]
mod additional_tests {
    use super::*;
    use crate::{NonStdLenUint, TryFromSlice};

    macro_rules! test_float {
        ($($name:ident: $ty:ty),* $(,)?) => {$(
            #[test]
            fn $name() -> Result<(), Box<dyn std::error::Error>> {
                let bytes = vec![255; std::mem::size_of::<$ty>()];
                let value = <$ty>::try_from_slice(bytes.as_slice(), &mut 0)?;
                assert!(value.is_missing());
                assert_eq!(value.to_bits(), <$ty>::missing().to_bits());
                assert!(!<$ty>::NAN.is_missing());
                assert!(!<$ty>::INFINITY.is_missing());
                assert!(!(0.0 as $ty).is_missing());
                Ok(())
            }
        )*};
    }
    test_float! { float32: f32, float64: f64 }

    macro_rules! test_width {
        ($($name:ident: $ty:ty),* $(,)?) => {$(
            #[test]
            fn $name() {
                let width = std::mem::size_of::<$ty>();
                for len in 1..=width {
                    let value = <$ty>::MAX >> ((width - len) * 8);
                    assert!(NonStdLenUint::new(value, len).is_missing());
                    assert!(!NonStdLenUint::new(value - 1, len).is_missing());
                }
                assert!(NonStdLenUint::<$ty>::missing().is_missing());
                assert!(!NonStdLenUint::new(0 as $ty, 0).is_missing());
                assert!(!NonStdLenUint::new(<$ty>::MAX, width + 1).is_missing());
            }
        )*};
    }
    test_width! { width8: u8, width16: u16, width32: u32, width64: u64 }
}
