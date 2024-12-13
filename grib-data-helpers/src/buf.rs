pub(crate) fn read_number<N>(buf: &[u8], pos: &mut usize) -> Result<N, &'static str>
where
    N: FromBytes,
{
    let start = *pos;
    *pos += std::mem::size_of::<N>();
    if *pos > (*buf).len() {
        return Err("reading a number failed");
    }
    let val = FromBytes::from_be_bytes(&buf[start..*pos]);
    Ok(val)
}

pub(crate) trait FromBytes {
    fn from_be_bytes(bytes: &[u8]) -> Self;
}

impl<const N: usize> FromBytes for [u8; N] {
    fn from_be_bytes(bytes: &[u8]) -> [u8; N] {
        // panics if N is larger than the slice length
        bytes[..N].try_into().unwrap()
    }
}

macro_rules! add_impl_for_types {
    ($($ty:ty,)*) => ($(
        impl FromBytes for $ty {
            fn from_be_bytes(bytes: &[u8]) -> $ty {
                <$ty>::from_be_bytes(FromBytes::from_be_bytes(bytes))
            }
        }
    )*);
}

add_impl_for_types![u8, u16, u32, u64,];
