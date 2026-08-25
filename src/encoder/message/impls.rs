use super::*;

impl<T: WriteGrib2Message + ?Sized> WriteGrib2Message for &T {
    type S1<'a>
        = <T as WriteGrib2Message>::S1<'a>
    where
        Self: 'a;

    type Item<'a>
        = <T as WriteGrib2Message>::Item<'a>
    where
        Self: 'a;

    type Iter<'a>
        = <T as WriteGrib2Message>::Iter<'a>
    where
        Self: 'a;

    fn discipline(&self) -> u8 {
        (*self).discipline()
    }

    fn section1(&self) -> &Self::S1<'_> {
        (*self).section1()
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
}

impl<T: WriteGrib2MessageIterL1 + ?Sized> WriteGrib2MessageIterL1 for &T {
    type S2<'a>
        = <T as WriteGrib2MessageIterL1>::S2<'a>
    where
        Self: 'a;

    type Item<'a>
        = <T as WriteGrib2MessageIterL1>::Item<'a>
    where
        Self: 'a;

    type Iter<'a>
        = <T as WriteGrib2MessageIterL1>::Iter<'a>
    where
        Self: 'a;

    fn section2(&self) -> Option<&Self::S2<'_>> {
        (*self).section2()
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
}

impl<T: WriteGrib2MessageIterL2 + ?Sized> WriteGrib2MessageIterL2 for &T {
    type S3<'a>
        = <T as WriteGrib2MessageIterL2>::S3<'a>
    where
        Self: 'a;

    type Item<'a>
        = <T as WriteGrib2MessageIterL2>::Item<'a>
    where
        Self: 'a;

    type Iter<'a>
        = <T as WriteGrib2MessageIterL2>::Iter<'a>
    where
        Self: 'a;

    fn section3(&self) -> &Self::S3<'_> {
        (*self).section3()
    }

    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
}

impl<T: WriteGrib2MessageIterL3 + ?Sized> WriteGrib2MessageIterL3 for &T {
    type S4<'a>
        = <T as WriteGrib2MessageIterL3>::S4<'a>
    where
        Self: 'a;

    type SD<'a>
        = <T as WriteGrib2MessageIterL3>::SD<'a>
    where
        Self: 'a;

    fn section4(&self) -> &Self::S4<'_> {
        (*self).section4()
    }

    fn data_sections(&self) -> &Self::SD<'_> {
        (*self).data_sections()
    }
}

macro_rules! add_impl_for_references {
    ($(($trait:path,$len_method:ident,$write_method:ident),)*) => ($(
        impl<T: $trait + ?Sized> $trait for &T {
            fn $len_method(&self) -> usize {
                (*self).$len_method()
            }

            fn $write_method(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                (*self).$write_method(buf)
            }
        }
    )*);
}

add_impl_for_references![
    (WriteGrib2Ident, section1_len, write_section1),
    (WriteGrib2LocalUse, section2_len, write_section2),
    (WriteGrib2GridDef, section3_len, write_section3),
    (WriteGrib2ProductDef, section4_len, write_section4),
    (
        WriteGrib2PointValues,
        data_sections_len,
        write_data_sections
    ),
];

impl<T: WriteGrib2DataSections + ?Sized> WriteGrib2DataSections for &T {
    fn section5_len(&self) -> usize {
        (*self).section5_len()
    }

    fn write_section5(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        (*self).write_section5(buf)
    }

    fn section6_len(&self) -> usize {
        (*self).section6_len()
    }

    fn write_section6(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        (*self).write_section6(buf)
    }

    fn section7_len(&self) -> usize {
        (*self).section7_len()
    }

    fn write_section7(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        (*self).write_section7(buf)
    }
}
