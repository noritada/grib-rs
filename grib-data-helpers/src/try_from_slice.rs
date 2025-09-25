use as_grib_int::AsGribInt;

pub fn read_from_slice<N>(slice: &[u8], pos: &mut usize) -> TryFromSliceResult<N>
where
    N: TryFromSlice,
{
    let start = *pos;
    *pos += std::mem::size_of::<N>();
    TryFromSlice::try_from_slice(&slice[start..*pos])
}

pub trait TryFromSlice {
    fn try_from_slice(slice: &[u8]) -> TryFromSliceResult<Self>
    where
        Self: Sized;
}

pub type TryFromSliceResult<T> = Result<T, &'static str>;

impl<const N: usize> TryFromSlice for [u8; N] {
    fn try_from_slice(slice: &[u8]) -> TryFromSliceResult<[u8; N]> {
        if N > slice.len() {
            Err("slice length is too short")
        } else {
            Ok(slice[..N].try_into().unwrap())
        }
    }
}

macro_rules! add_impl_for_unsigned_integer_and_float_types {
    ($($ty:ty,)*) => ($(
        impl TryFromSlice for $ty {
            fn try_from_slice(slice: &[u8]) -> TryFromSliceResult<$ty> {
                let n = <$ty>::from_be_bytes(TryFromSlice::try_from_slice(slice)?);
                Ok(n)
            }
        }
    )*);
}

add_impl_for_unsigned_integer_and_float_types![u8, u16, u32, u64, f32, f64,];

macro_rules! add_impl_for_signed_integer_types {
    ($(($ty_src:ty, $ty_dst:ty),)*) => ($(
        impl TryFromSlice for $ty_dst {
            fn try_from_slice(slice: &[u8]) -> TryFromSliceResult<$ty_dst> {
                let n = <$ty_src>::from_be_bytes(TryFromSlice::try_from_slice(slice)?).as_grib_int();
                Ok(n)
            }
        }
    )*);
}

add_impl_for_signed_integer_types![(u8, i8), (u16, i16), (u32, i32), (u64, i64),];

mod as_grib_int;
