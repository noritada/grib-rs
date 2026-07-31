mod multigrid;
mod multiproduct;
mod single;

pub use multigrid::*;
pub use multiproduct::*;
pub use single::*;

use crate::{Encoder, WriteToBuffer};

const SECT0_LEN: usize = 16;

pub trait WriteGrib2Message {
    type S1<'a>: WriteGrib2Ident
    where
        Self: 'a;

    type Item<'a>: WriteGrib2SubmessageL1
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = Self::Item<'a>>
    where
        Self: 'a;

    fn discipline(&self) -> u8;
    fn reserved(&self) -> [u8; 2] {
        [0xff, 0xff]
    }
    fn section1(&self) -> &Self::S1<'_>;
    fn iter(&self) -> Self::Iter<'_>;

    fn num_octets(&self) -> usize {
        SECT0_LEN
            + self.section1().section1_len()
            + self.iter().map(|m| m.num_octets()).sum::<usize>()
            + crate::SECT8_ES_SIZE
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let total_len = self.num_octets();
        if buf.len() < total_len {
            return Err("destination buffer is too small");
        }

        let mut pos = 0;
        pos += write_section0(
            self.discipline(),
            self.reserved(),
            total_len,
            &mut buf[pos..],
        )?;
        pos += self.section1().write_section1(&mut buf[pos..])?;
        for m in self.iter() {
            pos += m.write(&mut buf[pos..])?;
        }
        pos += write_section8(&mut buf[pos..])?;
        Ok(pos)
    }
}

pub trait WriteGrib2SubmessageL1 {
    type S2<'a>: WriteGrib2LocalUse
    where
        Self: 'a;

    type Item<'a>: WriteGrib2SubmessageL2
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = Self::Item<'a>>
    where
        Self: 'a;

    fn section2(&self) -> Option<&Self::S2<'_>>;
    fn iter(&self) -> Self::Iter<'_>;

    fn num_octets(&self) -> usize {
        self.section2().map_or(0, |m| m.section2_len())
            + self.iter().map(|m| m.num_octets()).sum::<usize>()
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let mut pos = 0;
        if let Some(section2) = self.section2() {
            pos += section2.write_section2(&mut buf[pos..])?;
        }
        for m in self.iter() {
            pos += m.write(&mut buf[pos..])?;
        }
        Ok(pos)
    }
}

pub trait WriteGrib2SubmessageL2 {
    type S3<'a>: WriteGrib2GridDef
    where
        Self: 'a;

    type Item<'a>: WriteGrib2SubmessageL3
    where
        Self: 'a;

    type Iter<'a>: Iterator<Item = Self::Item<'a>>
    where
        Self: 'a;

    fn section3(&self) -> &Self::S3<'_>;
    fn iter(&self) -> Self::Iter<'_>;

    fn num_octets(&self) -> usize {
        self.section3().section3_len() + self.iter().map(|m| m.num_octets()).sum::<usize>()
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let mut pos = 0;
        pos += self.section3().write_section3(&mut buf[pos..])?;
        for m in self.iter() {
            pos += m.write(&mut buf[pos..])?;
        }
        Ok(pos)
    }
}

pub trait WriteGrib2SubmessageL3 {
    type S4<'a>: WriteGrib2ProductDef
    where
        Self: 'a;
    type SD<'a>: WriteGrib2PointValues
    where
        Self: 'a;

    fn section4(&self) -> &Self::S4<'_>;
    fn data_sections(&self) -> &Self::SD<'_>;

    fn num_octets(&self) -> usize {
        self.section4().section4_len() + self.data_sections().data_sections_len()
    }

    fn write(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let mut pos = 0;
        pos += self.section4().write_section4(&mut buf[pos..])?;
        pos += self.data_sections().write_data_sections(&mut buf[pos..])?;
        Ok(pos)
    }
}

pub trait WriteGrib2Ident {
    fn section1_len(&self) -> usize;

    fn write_section1(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

pub trait WriteGrib2LocalUse {
    fn section2_len(&self) -> usize;

    fn write_section2(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

pub trait WriteGrib2GridDef {
    fn section3_len(&self) -> usize;

    fn write_section3(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

pub trait WriteGrib2ProductDef {
    fn section4_len(&self) -> usize;

    fn write_section4(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

pub trait WriteGrib2PointValues {
    fn data_sections_len(&self) -> usize;

    fn write_data_sections(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

macro_rules! add_impl_for_u8_slices {
    ($(($trait:ty,$len_method:ident,$write_method:ident,$sect_num:expr),)*) => ($(
        impl<T: AsRef<[u8]>> $trait for T {
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
];

/// A serializer that writes the byte sequence of sections concerning GPV data
/// to the output buffer.
pub trait WriteGrib2DataSections {
    /// Returns the length of the byte sequence in Section 5.
    fn section5_len(&self) -> usize;

    /// Writes the byte sequence of Section 5 to the output buffer.
    fn write_section5(&self, buf: &mut [u8]) -> Result<usize, &'static str>;

    /// Returns the length of the byte sequence in Section 6.
    fn section6_len(&self) -> usize;

    /// Writes the byte sequence of Section 6 to the output buffer.
    fn write_section6(&self, buf: &mut [u8]) -> Result<usize, &'static str>;

    /// Returns the length of the byte sequence in Section 7.
    fn section7_len(&self) -> usize;

    /// Writes the byte sequence of Section 7 to the output buffer.
    fn write_section7(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

fn write_section0(
    discipline: u8,
    reserved: [u8; 2],
    len: usize,
    buf: &mut [u8],
) -> Result<usize, &'static str> {
    if buf.len() < SECT0_LEN {
        return Err("destination buffer is too small");
    }

    let sect = crate::def::grib2::Section0 {
        identifier: [0x47, 0x52, 0x49, 0x42],
        reserved,
        discipline,
        edition_num: 2,
        total_len: len as u64,
    };

    let mut pos = 0;
    pos += sect.write_to_buffer(&mut buf[pos..])?;
    Ok(pos)
}

fn write_section8(buf: &mut [u8]) -> Result<usize, &'static str> {
    const SIGNATURE: [u8; 4] = [0x37, 0x37, 0x37, 0x37];
    if buf.len() < SIGNATURE.num_bytes_required() {
        return Err("destination buffer is too small");
    }
    SIGNATURE.write_to_buffer(buf)
}

pub(crate) fn write_section_header(
    len: u32,
    sect_num: u8,
    buf: &mut [u8],
) -> Result<usize, &'static str> {
    crate::def::grib2::SectionHeader { len, sect_num }.write_to_buffer(buf)
}

impl<'a, 'd, P> WriteGrib2SubmessageL3 for (&'a P, &'a Encoder<'d>)
where
    P: WriteGrib2ProductDef,
{
    type S4<'s>
        = P
    where
        Self: 's;

    type SD<'s>
        = Encoder<'d>
    where
        Self: 's;

    fn section4(&self) -> &Self::S4<'_> {
        self.0
    }

    fn data_sections(&self) -> &Self::SD<'_> {
        self.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TryFromSlice as _, def::grib2::Section1};

    #[test]
    fn grib2_section1_roundtrip_test() -> Result<(), Box<dyn std::error::Error>> {
        let sect = Section1 {
            header: crate::def::grib2::SectionHeader {
                len: 21,
                sect_num: 1,
            },
            payload: crate::def::grib2::Section1Payload {
                centre_id: 0xffff,
                subcentre_id: 0,
                master_table_version: 29,
                local_table_version: 0,
                ref_time_significance: 0,
                ref_time: crate::def::grib2::RefTime {
                    year: 2026,
                    month: 1,
                    day: 2,
                    hour: 3,
                    minute: 4,
                    second: 5,
                },
                prod_status: 0,
                data_type: 0,
                optional: None,
            },
        };
        let mut buf = vec![0; 21];
        sect.payload.write_section1(&mut buf)?;
        let decoded = Section1::try_from_slice(&buf, &mut 0)?;
        assert_eq!(decoded, sect);
        Ok(())
    }
}
