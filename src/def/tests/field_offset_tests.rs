use crate::{
    FieldOffset, ReadFixedField, TryFromSlice,
    def::grib2::template::{
        Template4_0, Template4_1, Template4_2, Template4_5, Template4_6, Template4_8, Template4_9,
        Template4_10, Template4_11, Template4_12, param_set,
    },
};

macro_rules! test_common_fields {
    ($($name:ident: $template:ty),* $(,)?) => {$(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            use param_set::{ForecastTime, Horizontal, ProductParam};

            assert_eq!(<$template as FieldOffset<ProductParam>>::OFFSET, 0);
            assert_eq!(<$template as FieldOffset<ForecastTime>>::OFFSET, 5);
            assert_eq!(<$template as FieldOffset<Horizontal>>::OFFSET, 13);
            let buf: Vec<u8> = (0..25).collect();
            assert_eq!(
                <$template as ReadFixedField<ProductParam>>::read_fixed_field(&buf[..2])?,
                ProductParam::try_from_slice(&buf, &mut 0)?,
            );
            assert_eq!(
                <$template as ReadFixedField<ForecastTime>>::read_fixed_field(&buf[..13])?,
                ForecastTime::try_from_slice(&buf, &mut 5)?,
            );
            assert_eq!(
                <$template as ReadFixedField<Horizontal>>::read_fixed_field(&buf)?,
                Horizontal::try_from_slice(&buf, &mut 13)?,
            );
            Ok(())
        }
    )*};
}

test_common_fields! {
    template4_0: Template4_0,
    template4_1: Template4_1,
    template4_2: Template4_2,
    template4_5: Template4_5,
    template4_6: Template4_6,
    template4_8: Template4_8,
    template4_9: Template4_9,
    template4_10: Template4_10,
    template4_11: Template4_11,
    template4_12: Template4_12,
}
