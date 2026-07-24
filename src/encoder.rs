pub use complex::*;
pub use simple::*;

use crate::{WriteToBuffer, def::grib2::template::param_set};

pub struct Encoder<'d> {
    data: std::borrow::Cow<'d, [f64]>,
    method: EncodingMethod,
}

impl<'d> Encoder<'d> {
    pub fn new(data: std::borrow::Cow<'d, [f64]>, method: EncodingMethod) -> Self {
        Self { data, method }
    }

    /// Encodes a sequence of numerical values as GRIB2 data sections.
    pub fn encode(&self) -> EncodeOutput {
        let output = match &self.method {
            EncodingMethod::SimplePacking(simple_packing_strategy) => {
                let encoder = simple::Encoder::new(&self.data, simple_packing_strategy.clone());
                EncodeOutputInner::SimplePacking(encoder.encode())
            }
            EncodingMethod::ComplexPacking(
                simple_packing_strategy,
                complex_packing_strategy,
                _spatial_differencing_option,
            ) => {
                let encoder = complex::Encoder::new(
                    &self.data,
                    simple_packing_strategy.clone(),
                    complex_packing_strategy.clone(),
                );
                EncodeOutputInner::ComplexPacking(encoder.encode())
            }
        };
        EncodeOutput(output)
    }

    pub fn encode_and_write_point_values(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        let encoded = self.encode();
        let mut pos = 0;
        pos += encoded.write_section5(&mut buf[pos..])?;
        pos += encoded.write_section6(&mut buf[pos..])?;
        pos += encoded.write_section7(&mut buf[pos..])?;
        Ok(pos)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum EncodingMethod {
    /// Simple packing.
    SimplePacking(SimplePackingStrategy),
    /// Complex packing.
    ComplexPacking(
        SimplePackingStrategy,
        ComplexPackingStrategy,
        SpatialDifferencingOption,
    ),
}

/// Data obtained through encoding. Instances are typically used to write GRIB2
/// data via the methods defined in [`WriteGrib2DataSections`].
#[derive(Debug)]
pub struct EncodeOutput(EncodeOutputInner);

impl EncodeOutput {
    /// Returns the parameter set.
    pub fn params(&self) -> EncodeOutputParams<'_> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => {
                EncodeOutputParams::SimplePacking(encoded.params())
            }
            EncodeOutputInner::ComplexPacking(encoded) => {
                let (simple, complex) = encoded.params();
                EncodeOutputParams::ComplexPacking(simple, complex)
            }
        }
    }
}

impl WriteGrib2DataSections for EncodeOutput {
    fn section5_len(&self) -> usize {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.section5_len(),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.section5_len(),
        }
    }

    fn write_section5(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.write_section5(buf),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.write_section5(buf),
        }
    }

    fn section6_len(&self) -> usize {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.section6_len(),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.section6_len(),
        }
    }

    fn write_section6(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.write_section6(buf),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.write_section6(buf),
        }
    }

    fn section7_len(&self) -> usize {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.section7_len(),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.section7_len(),
        }
    }

    fn write_section7(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
        match &self.0 {
            EncodeOutputInner::SimplePacking(encoded) => encoded.write_section7(buf),
            EncodeOutputInner::ComplexPacking(encoded) => encoded.write_section7(buf),
        }
    }
}

#[non_exhaustive]
pub enum EncodeOutputParams<'a> {
    SimplePacking(&'a param_set::SimplePacking),
    ComplexPacking(&'a param_set::SimplePacking, &'a param_set::ComplexPacking),
}

#[derive(Debug)]
enum EncodeOutputInner {
    SimplePacking(simple::Encoded),
    ComplexPacking(complex::Encoded),
}

trait Encode {
    type Output;

    fn encode(&self) -> Self::Output;
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

macro_rules! add_impl_for_u8_slices {
    ($(($trait:ty,$len_method:ident,$write_method:ident,$sect_num:expr),)*) => ($(
        impl $trait for &[u8] {
            fn $len_method(&self) -> usize {
                self.len() + 5
            }

            fn $write_method(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                let len = self.$len_method();
                if buf.len() < len {
                    return Err("destination buffer is too small");
                }

                let mut pos = 0;
                pos += write_section_header(len as u32, $sect_num, &mut buf[pos..])?;
                buf[pos..len].copy_from_slice(self);
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

pub fn write_section0(discipline: u8, len: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    const LEN: usize = 16;
    if buf.len() < LEN {
        return Err("destination buffer is too small");
    }

    let sect = crate::def::grib2::Section0 {
        identifier: [0x47, 0x52, 0x49, 0x42],
        reserved: [0xff, 0xff],
        discipline,
        edition_num: 2,
        total_len: len as u64,
    };

    let mut pos = 0;
    pos += sect.write_to_buffer(&mut buf[pos..])?;
    Ok(pos)
}

pub fn write_section8(buf: &mut [u8]) -> Result<usize, &'static str> {
    const SIGNATURE: [u8; 4] = [0x37, 0x37, 0x37, 0x37];
    if buf.len() < SIGNATURE.num_bytes_required() {
        return Err("destination buffer is too small");
    }
    SIGNATURE.write_to_buffer(buf)
}

fn write_section_header(len: u32, sect_num: u8, buf: &mut [u8]) -> Result<usize, &'static str> {
    crate::def::grib2::SectionHeader { len, sect_num }.write_to_buffer(buf)
}

mod bitmap;
mod complex;
mod helpers;
mod simple;
mod writer;

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
