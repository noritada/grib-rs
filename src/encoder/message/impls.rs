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

macro_rules! add_impl_for_u8_slices {
    ($(($trait:ty,$len_method:ident,$write_method:ident,$sect_num:expr),)*) => ($(
        impl $trait for [u8] {
            fn $len_method(&self) -> usize {
                self.as_ref().len() + 5
            }

            fn $write_method(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                let len = self.$len_method();
                if buf.len() < len {
                    return Err("destination buffer is too small");
                }

                let mut pos = 0;
                pos += write_section_header(len as u32, $sect_num, &mut buf[pos..])?;
                buf[pos..len].copy_from_slice(self.as_ref());
                Ok(len)
            }
        }
    )*);
}

add_impl_for_u8_slices![
    (WriteGrib2Ident, section1_len, write_section1, 1),
    (WriteGrib2LocalUse, section2_len, write_section2, 2),
    (WriteGrib2GridDef, section3_len, write_section3, 3),
    (WriteGrib2ProductDef, section4_len, write_section4, 4),
];

macro_rules! add_impl_for_payload_structs {
    ($(($trait:ty,$ty:ty,$len_method:ident,$write_method:ident,$sect_num:expr),)*) => ($(
        impl $trait for $ty {
            fn $len_method(&self) -> usize {
                self.num_bytes_required() + 5
            }

            fn $write_method(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                let len = self.$len_method();
                if buf.len() < len {
                    return Err("destination buffer is too small");
                }

                let mut pos = 0;
                pos += write_section_header(len as u32, $sect_num, &mut buf[pos..])?;
                pos += self.write_to_buffer(&mut buf[pos..])?;
                Ok(pos)
            }
        }
    )*);
}

add_impl_for_payload_structs![
    (
        WriteGrib2Ident,
        crate::def::grib2::Section1Payload,
        section1_len,
        write_section1,
        1
    ),
    (
        WriteGrib2GridDef,
        crate::def::grib2::Section3Payload,
        section3_len,
        write_section3,
        3
    ),
    (
        WriteGrib2ProductDef,
        crate::def::grib2::Section4Payload,
        section4_len,
        write_section4,
        4
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

impl<'a, 'd, P> WriteGrib2MessageIterL3 for (&'a P, &'a GpvEncoder<'d>)
where
    P: WriteGrib2ProductDef,
{
    type S4<'s>
        = P
    where
        Self: 's;

    type SD<'s>
        = GpvEncoder<'d>
    where
        Self: 's;

    fn section4(&self) -> &Self::S4<'_> {
        self.0
    }

    fn data_sections(&self) -> &Self::SD<'_> {
        self.1
    }
}
