pub use complex::*;
use grib_template_helpers::WriteToBuffer;
pub use simple::*;

/// An encoder that encodes a sequence of numerical values as GRIB2 data
/// sections
pub trait Encode {
    type Output;

    /// Performs data encoding.
    fn encode(&self) -> Self::Output;
}

/// A serializer that writes the byte sequence of sections concerning GPV data
/// to the output buffer
pub trait WriteGrib2DataSections {
    /// Returns the length of the byte sequence in Section 5.
    fn section5_len(&self) -> usize;

    /// Writes the byte sequence of Section 5 to the output buffer
    fn write_section5(&self, buf: &mut [u8]) -> Result<usize, &'static str>;

    /// Returns the length of the byte sequence in Section 6.
    fn section6_len(&self) -> usize;

    /// Writes the byte sequence of Section 6 to the output buffer
    fn write_section6(&self, buf: &mut [u8]) -> Result<usize, &'static str>;

    /// Returns the length of the byte sequence in Section 7.
    fn section7_len(&self) -> usize;

    /// Writes the byte sequence of Section 7 to the output buffer
    fn write_section7(&self, buf: &mut [u8]) -> Result<usize, &'static str>;
}

pub fn write_section0(discipline: u8, len: usize, buf: &mut [u8]) -> Result<usize, &'static str> {
    const HEAD: [u8; 6] = [0x47, 0x52, 0x49, 0x42, 0xff, 0xff];
    const EDITION: u8 = 2;
    const LEN: usize = 16;
    if buf.len() < LEN {
        return Err("destination buffer is too small");
    }

    let mut pos = 0;
    pos += HEAD.write_to_buffer(&mut buf[pos..])?;
    pos += discipline.write_to_buffer(&mut buf[pos..])?;
    pos += EDITION.write_to_buffer(&mut buf[pos..])?;
    pos += (len as u64).write_to_buffer(&mut buf[pos..])?;
    Ok(pos)
}

pub fn write_section1(
    payload: &crate::def::grib2::Section1Payload,
    buf: &mut [u8],
) -> Result<usize, &'static str> {
    const LEN: usize = 0x15;
    if buf.len() < LEN {
        return Err("destination buffer is too small");
    }

    let mut pos = 0;
    pos += (LEN as u32).write_to_buffer(&mut buf[pos..])?;
    pos += 1_u8.write_to_buffer(&mut buf[pos..])?;
    pos += payload.write_to_buffer(&mut buf[pos..])?;
    Ok(pos)
}

pub fn write_section8(buf: &mut [u8]) -> Result<usize, &'static str> {
    const SIGNATURE: [u8; 4] = [0x37, 0x37, 0x37, 0x37];
    if buf.len() < SIGNATURE.num_bytes_required() {
        return Err("destination buffer is too small");
    }
    SIGNATURE.write_to_buffer(buf)
}

mod complex;
mod helpers;
mod simple;
mod writer;

#[cfg(test)]
mod tests {
    use grib_template_helpers::TryFromSlice as _;

    use super::*;
    use crate::def::grib2::Section1;

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
        write_section1(&sect.payload, &mut buf)?;
        let decoded = Section1::try_from_slice(&buf, &mut 0)?;
        assert_eq!(decoded, sect);
        Ok(())
    }
}
