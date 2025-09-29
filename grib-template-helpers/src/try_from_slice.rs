use as_grib_signed::AsGribSigned;

pub trait TryFromSlice {
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

pub trait TryEnumFromSlice {
    fn try_enum_from_slice(
        discriminant: impl Into<u64>,
        slice: &[u8],
        pos: &mut usize,
    ) -> TryFromSliceResult<Self>
    where
        Self: Sized;
}

mod as_grib_signed;
