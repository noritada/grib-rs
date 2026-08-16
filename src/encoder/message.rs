pub use multigrid::*;
pub use multiproduct::*;
pub use single::*;

use super::GpvEncoder;
use crate::WriteToBuffer;

const SECT0_LEN: usize = 16;

/// A functionality to write an entire GRIB2 message. This trait works in
/// conjunction with [`WriteGrib2MessageIterL1`], [`WriteGrib2MessageIterL2`],
/// and [`WriteGrib2MessageIterL3`] to write the message.
pub trait WriteGrib2Message {
    type S1<'a>: WriteGrib2Ident
    where
        Self: 'a;

    type Item<'a>: WriteGrib2MessageIterL1
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

/// A functionality to write elements of the L1 iterator in a GRIB2 message.
/// This trait works in conjunction with [`WriteGrib2Message`],
/// [`WriteGrib2MessageIterL2`], and [`WriteGrib2MessageIterL3`] to write the
/// message.
pub trait WriteGrib2MessageIterL1 {
    type S2<'a>: WriteGrib2LocalUse
    where
        Self: 'a;

    type Item<'a>: WriteGrib2MessageIterL2
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

/// A functionality to write elements of the L2 iterator in a GRIB2 message.
/// This trait works in conjunction with [`WriteGrib2Message`],
/// [`WriteGrib2MessageIterL1`], and [`WriteGrib2MessageIterL3`] to write the
/// message.
pub trait WriteGrib2MessageIterL2 {
    type S3<'a>: WriteGrib2GridDef
    where
        Self: 'a;

    type Item<'a>: WriteGrib2MessageIterL3
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

/// A functionality to write elements of the L3 iterator in a GRIB2 message.
/// This trait works in conjunction with [`WriteGrib2Message`],
/// [`WriteGrib2MessageIterL1`], and [`WriteGrib2MessageIterL2`] to write the
/// message.
pub trait WriteGrib2MessageIterL3 {
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

/// A functionality to write the byte sequence of Section 1 (Identification
/// Section) of a GRIB2 message.
///
/// # Examples
///
/// This trait is implemented for the payload struct of Section 1
/// ([`def::grib2::Section1Payload`](crate::def::grib2::Section1Payload)).
///
/// ```
/// use grib::{TryFromSlice, def::grib2, encoder::WriteGrib2Ident};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let sect = grib2::Section1 {
///         header: grib2::SectionHeader {
///             len: 21,
///             sect_num: 1,
///         },
///         payload: grib2::Section1Payload {
///             centre_id: 0xffff,
///             subcentre_id: 0,
///             master_table_version: 29,
///             local_table_version: 0,
///             ref_time_significance: 0,
///             ref_time: grib2::template::param_set::DateTime {
///                 year: 2026,
///                 month: 1,
///                 day: 2,
///                 hour: 3,
///                 minute: 4,
///                 second: 5,
///             },
///             prod_status: 0,
///             data_type: 0,
///             optional: None,
///         },
///     };
///     let mut buf = vec![0; sect.payload.section1_len()];
///     sect.payload.write_section1(&mut buf)?;
///     let decoded = grib2::Section1::try_from_slice(&buf, &mut 0);
///     assert_eq!(decoded, Ok(sect));
///
///     Ok(())
/// }
/// ```
///
/// This trait is also implemented for the byte sequence of the payload of
/// Section 1.
///
/// ```
/// use grib::{TryFromSlice, def::grib2, encoder::WriteGrib2Ident};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut input = [
///         0xff, 0xff, 0x00, 0x00, 0x1d, 0x00, 0x00, 0x07, 0xea, 0x01, 0x02, 0x03, 0x04, 0x05,
///         0x00, 0x00,
///     ];
///     let mut buf = vec![0; input.section1_len()];
///     input.write_section1(&mut buf)?;
///
///     // The output is simply the input byte sequence with a section header appended to it.
///     assert_eq!(&buf[5..], input);
///
///     let decoded = grib2::Section1::try_from_slice(&buf, &mut 0);
///     let expected = Ok(grib2::Section1 {
///         header: grib2::SectionHeader {
///             len: 21,
///             sect_num: 1,
///         },
///         payload: grib2::Section1Payload {
///             centre_id: 0xffff,
///             subcentre_id: 0,
///             master_table_version: 29,
///             local_table_version: 0,
///             ref_time_significance: 0,
///             ref_time: grib2::template::param_set::DateTime {
///                 year: 2026,
///                 month: 1,
///                 day: 2,
///                 hour: 3,
///                 minute: 4,
///                 second: 5,
///             },
///             prod_status: 0,
///             data_type: 0,
///             optional: None,
///         },
///     });
///     assert_eq!(decoded, expected);
///
///     Ok(())
/// }
/// ```
///
/// Since no constraints can be placed on the byte sequence, `write_section1`
/// can, of course, be executed on any byte sequence; however, if it is executed
/// on an inappropriate byte sequence, the resulting byte sequence will not be
/// correctly interpreted as Section 1.
///
/// ```
/// use grib::{TryFromSlice, def::grib2, encoder::WriteGrib2Ident};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut input = [0xff, 0xff];
///     let mut buf = vec![0; input.section1_len()];
///     input.write_section1(&mut buf)?;
///
///     // The output is simply the input byte sequence with a section header appended to it.
///     assert_eq!(&buf[5..], input);
///
///     // The output cannot be correctly interpreted as Section 1.
///     let decoded = grib2::Section1::try_from_slice(&buf, &mut 0);
///     let expected = Err("slice length is too short");
///     assert_eq!(decoded, expected);
///
///     Ok(())
/// }
/// ```
pub trait WriteGrib2Ident {
    fn section1_len(&self) -> usize;

    fn write_section1(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

/// A functionality to write the byte sequence of Section 2 (Local Use Section)
/// of a GRIB2 message.
pub trait WriteGrib2LocalUse {
    fn section2_len(&self) -> usize;

    fn write_section2(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

/// A functionality to write the byte sequence of Section 3 (Grid Definition
/// Section) of a GRIB2 message.
pub trait WriteGrib2GridDef {
    fn section3_len(&self) -> usize;

    fn write_section3(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

/// A functionality to write the byte sequence of Section 4 (Product Definition
/// Section) of a GRIB2 message.
pub trait WriteGrib2ProductDef {
    fn section4_len(&self) -> usize;

    fn write_section4(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

/// A functionality to write the byte sequence of Section 5 (Data
/// Representation Section), Section 6 (Bit-map section), and Section 7 (Data
/// Section) of a GRIB2 message.
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
    (
        WriteGrib2ProductDef,
        crate::def::grib2::Section4Payload,
        section4_len,
        write_section4,
        4
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

mod multigrid;
mod multiproduct;
mod single;
