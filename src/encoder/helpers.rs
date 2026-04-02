pub(crate) trait BitsRequired {
    fn bits_required(&self) -> u8;
}

macro_rules! add_impl_for_integer_types {
    ($($ty:ty,)*) => ($(
        impl BitsRequired for $ty {
            fn bits_required(&self) -> u8 {
                (Self::BITS - self.leading_zeros()) as u8
            }
        }
    )*);
}

add_impl_for_integer_types![u32, usize,];

impl BitsRequired for f64 {
    fn bits_required(&self) -> u8 {
        (self + 1.).log2().ceil() as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::encoder::helpers::BitsRequired;

    #[test]
    fn bits_calculation() {
        assert_eq!(0_u32.bits_required(), 0);
        assert_eq!(1_u32.bits_required(), 1);
        assert_eq!(2_u32.bits_required(), 2);
        assert_eq!(3_u32.bits_required(), 2);
        assert_eq!(4_u32.bits_required(), 3);
        assert_eq!(8_u32.bits_required(), 4);

        assert_eq!(0_f64.bits_required(), 0);
        assert_eq!(1_f64.bits_required(), 1);
        assert_eq!(2_f64.bits_required(), 2);
        assert_eq!(3_f64.bits_required(), 2);
        assert_eq!(4_f64.bits_required(), 3);
        assert_eq!(8_f64.bits_required(), 4);
    }
}
