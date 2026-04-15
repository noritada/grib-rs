pub(crate) trait ToGribSigned<I> {
    fn to_grib_signed(&self) -> I;
}

macro_rules! add_impl_for_integer_types {
    ($(($ty_src:ty, $ty_dst:ty),)*) => ($(
        impl ToGribSigned<$ty_dst> for $ty_src {
            fn to_grib_signed(&self) -> $ty_dst {
                if self.is_negative() {
                    let abs = -self as $ty_dst;
                    abs | (1 << (Self::BITS - 1))
                } else {
                    *self as $ty_dst
                }
            }
        }
    )*);
}

add_impl_for_integer_types! {
    (i8, u8),
    (i16, u16),
    (i32, u32),
    (i64, u64),
}
