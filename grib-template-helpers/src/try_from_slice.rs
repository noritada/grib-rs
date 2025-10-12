use as_grib_signed::AsGribSigned;

/// A deserializer that reads a slice and stores output data in a struct.
///
/// # Examples
///
/// ```
/// use grib_template_helpers::{TryFromSlice, TryFromSliceResult};
///
/// #[derive(Debug, PartialEq, Eq)]
/// struct VariableLength {
///     len: u8,
///     seq: Vec<u8>,
/// }
///
/// impl TryFromSlice for VariableLength {
///     fn try_from_slice(slice: &[u8], pos: &mut usize) -> TryFromSliceResult<Self> {
///         if slice.len() == 0 {
///             return Err("too short slice");
///         }
///         let len = slice[*pos];
///         *pos += 1;
///
///         let end = *pos + usize::from(len);
///         if slice.len() < end {
///             return Err("too short slice");
///         }
///         let seq = slice[*pos..end].to_vec();
///         Ok(Self { len, seq })
///     }
/// }
///
/// let mut pos = 0;
/// let actual = VariableLength::try_from_slice(&[0], &mut pos);
/// let expected = Ok(VariableLength {
///     len: 0,
///     seq: Vec::new(),
/// });
/// assert_eq!(actual, expected);
///
/// let mut pos = 0;
/// let actual = VariableLength::try_from_slice(&[3, 1, 2, 3], &mut pos);
/// let expected = Ok(VariableLength {
///     len: 3,
///     seq: vec![1, 2, 3],
/// });
/// assert_eq!(actual, expected);
/// ```
pub trait TryFromSlice {
    /// Performs reading. The `pos` argument is a variable storing the starting
    /// position for reading within the slice.
    ///
    /// As reading proceeds, this `pos` changes, allowing the user to track how
    /// far they have read.
    fn try_from_slice(slice: &[u8], pos: &mut usize) -> TryFromSliceResult<Self>
    where
        Self: Sized;
}

pub type TryFromSliceResult<T> = Result<T, &'static str>;

impl<const N: usize> TryFromSlice for [u8; N] {
    fn try_from_slice(slice: &[u8], pos: &mut usize) -> TryFromSliceResult<[u8; N]> {
        let start = *pos;
        *pos += N;
        if *pos > slice.len() {
            Err("slice length is too short")
        } else {
            Ok(slice[start..*pos].try_into().unwrap())
        }
    }
}

macro_rules! add_impl_for_unsigned_integer_and_float_types {
    ($($ty:ty,)*) => ($(
        impl TryFromSlice for $ty {
            fn try_from_slice(slice: &[u8], pos: &mut usize) -> TryFromSliceResult<$ty> {
                let n = <$ty>::from_be_bytes(TryFromSlice::try_from_slice(slice, pos)?);
                Ok(n)
            }
        }
    )*);
}

add_impl_for_unsigned_integer_and_float_types![u8, u16, u32, u64, f32, f64,];

macro_rules! add_impl_for_signed_integer_types {
    ($(($ty_src:ty, $ty_dst:ty),)*) => ($(
        impl TryFromSlice for $ty_dst {
            fn try_from_slice(slice: &[u8], pos: &mut usize) -> TryFromSliceResult<$ty_dst> {
                let n = <$ty_src>::from_be_bytes(TryFromSlice::try_from_slice(slice, pos)?)
                    .as_grib_signed();
                Ok(n)
            }
        }
    )*);
}

add_impl_for_signed_integer_types![(u8, i8), (u16, i16), (u32, i32), (u64, i64),];

impl<T: TryFromSlice> TryFromSlice for Option<T> {
    fn try_from_slice(slice: &[u8], pos: &mut usize) -> TryFromSliceResult<Option<T>> {
        let result = if *pos == slice.len() {
            None
        } else {
            Some(T::try_from_slice(slice, pos)?)
        };
        Ok(result)
    }
}

pub trait TryEnumFromSlice {
    fn try_enum_from_slice(
        discriminant: impl Into<u64>,
        slice: &[u8],
        pos: &mut usize,
    ) -> TryFromSliceResult<Self>
    where
        Self: Sized;
}

impl<T: TryEnumFromSlice> TryEnumFromSlice for Option<T> {
    fn try_enum_from_slice(
        discriminant: impl Into<u64>,
        slice: &[u8],
        pos: &mut usize,
    ) -> TryFromSliceResult<Option<T>> {
        let result = if *pos == slice.len() {
            None
        } else {
            Some(T::try_enum_from_slice(discriminant, slice, pos)?)
        };
        Ok(result)
    }
}

mod as_grib_signed;
