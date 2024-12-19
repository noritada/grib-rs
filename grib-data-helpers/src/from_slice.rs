use as_grib_int::AsGribInt;

pub fn read_from_slice<N>(slice: &[u8], pos: &mut usize) -> Result<N, &'static str>
where
    N: FromSlice,
{
    let start = *pos;
    *pos += std::mem::size_of::<N>();
    if *pos > (*slice).len() {
        return Err("reading from slice failed");
    }
    let val = FromSlice::from_slice(&slice[start..*pos]);
    Ok(val)
}

pub trait FromSlice {
    fn from_slice(slice: &[u8]) -> Self;
}

impl<const N: usize> FromSlice for [u8; N] {
    fn from_slice(bytes: &[u8]) -> [u8; N] {
        // panics if N is larger than the slice length
        bytes[..N].try_into().unwrap()
    }
}

macro_rules! add_impl_for_unsigned_integer_types {
    ($($ty:ty,)*) => ($(
        impl FromSlice for $ty {
            fn from_slice(slice: &[u8]) -> $ty {
                <$ty>::from_be_bytes(FromSlice::from_slice(slice))
            }
        }
    )*);
}

add_impl_for_unsigned_integer_types![u8, u16, u32, u64,];

macro_rules! add_impl_for_signed_integer_types {
    ($(($ty_src:ty, $ty_dst:ty),)*) => ($(
        impl FromSlice for $ty_dst {
            fn from_slice(slice: &[u8]) -> $ty_dst {
                <$ty_src>::from_be_bytes(FromSlice::from_slice(slice)).as_grib_int()
            }
        }
    )*);
}

add_impl_for_signed_integer_types![(u8, i8), (u16, i16), (u32, i32), (u64, i64),];

mod as_grib_int;
