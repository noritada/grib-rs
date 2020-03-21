use std::convert::TryInto;
use std::io;
use std::io::Read;
use std::result::Result;

const SECT_HEADER_SIZE: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionInfo {
    pub num: u8,
    pub size: usize,
    pub body: Option<SectionBody>,
}

impl SectionInfo {
    pub fn read_body<R: Read>(&self, mut f: &mut R) -> Result<SectionBody, ParseError> {
        let body_size = self.size - SECT_HEADER_SIZE;
        let body = match self.num {
            1 => unpack_sect1_body(&mut f, body_size)?,
            2 => unpack_sect2_body(&mut f, body_size)?,
            3 => unpack_sect3_body(&mut f, body_size)?,
            4 => unpack_sect4_body(&mut f, body_size)?,
            5 => unpack_sect5_body(&mut f, body_size)?,
            6 => unpack_sect6_body(&mut f, body_size)?,
            7 => unpack_sect7_body(&mut f, body_size)?,
            _ => return Err(ParseError::UnknownSectionNumber(self.num)),
        };
        Ok(body)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionBody {
    Section1 {
        /// GRIB Master Tables Version Number
        master_table_version: u8,
        /// GRIB Local Tables Version Number
        local_table_version: u8,
        /// Significance of Reference Time
        ref_time: RefTime,
    },
    Section2,
    Section3 {
        /// Number of data points
        num_points: u32,
        /// Grid Definition Template Number
        grid_tmpl_num: u16,
    },
    Section4 {
        /// Number of coordinate values after Template
        num_coordinates: u16,
        /// Product Definition Template Number
        prod_tmpl_num: u16,
    },
    Section5 {
        /// Number of data points where one or more values are
        /// specified in Section 7 when a bit map is present, total
        /// number of data points when a bit map is absent
        num_points: u32,
        /// Data Representation Template Number
        repr_tmpl_code: u16,
    },
    Section6 {
        /// Bit-map indicator
        bitmap_indicator: u8,
    },
    Section7,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefTime {
    pub year: u16,
    pub month: u8,
    pub date: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseError {
    ReadError(String),
    NotGRIB,
    GRIBVersionMismatch(u8),
    UnknownSectionNumber(u8),
    EndSectionMismatch,
}

macro_rules! read_as {
    ($ty:ty, $name:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($name[$start..end].try_into().unwrap())
    }};
}

pub fn unpack_sect0<R: Read>(f: &mut R) -> Result<usize, ParseError> {
    let magic = b"GRIB";
    let mut buf = [0; 16];
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    if &buf[0..4] != magic {
        return Err(ParseError::NotGRIB);
    }
    let version = buf[7];
    if version != 2 {
        return Err(ParseError::GRIBVersionMismatch(version));
    }

    let fsize = read_as!(u64, buf, 8);

    Ok(fsize as usize)
}

pub fn unpack_sect1_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 16]; // octet 6-21
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..]).map_err(clarify_err)?;
    }

    Ok(SectionBody::Section1 {
        master_table_version: buf[4],
        local_table_version: buf[5],
        ref_time: RefTime {
            year: read_as!(u16, buf, 7),
            month: buf[9],
            date: buf[10],
            hour: buf[11],
            minute: buf[12],
            second: buf[13],
        },
    })
}

pub fn unpack_sect2_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let len_extra = body_size;
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..]).map_err(clarify_err)?;
    }

    Ok(SectionBody::Section2)
}

pub fn unpack_sect3_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 9]; // octet 6-14
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..]).map_err(clarify_err)?;
    }

    Ok(SectionBody::Section3 {
        num_points: read_as!(u32, buf, 1),
        grid_tmpl_num: read_as!(u16, buf, 7),
    })
}

pub fn unpack_sect4_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 4]; // octet 6-9
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..]).map_err(clarify_err)?;
    }

    Ok(SectionBody::Section4 {
        num_coordinates: read_as!(u16, buf, 0),
        prod_tmpl_num: read_as!(u16, buf, 2),
    })
}

pub fn unpack_sect5_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 6]; // octet 6-11
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..]).map_err(clarify_err)?;
    }

    Ok(SectionBody::Section5 {
        num_points: read_as!(u32, buf, 0),
        repr_tmpl_code: read_as!(u16, buf, 4),
    })
}

pub fn unpack_sect6_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 1]; // octet 6
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..]).map_err(clarify_err)?;
    }

    Ok(SectionBody::Section6 {
        bitmap_indicator: buf[0],
    })
}

pub fn unpack_sect7_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let len_extra = body_size;
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra]; // octet 6-21
        f.read_exact(&mut buf[..]).map_err(clarify_err)?;
    }

    Ok(SectionBody::Section7)
}

pub fn unpack_sect8<R: Read>(f: &mut R) -> Result<(), ParseError> {
    let magic = b"7777";
    let mut buf = magic.clone();
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    if buf[..] != magic[..] {
        return Err(ParseError::EndSectionMismatch);
    }

    Ok(())
}

/// Reads a common header for sections 1-7 and returns the section
/// number and size.
pub fn unpack_sect_header<R: Read>(f: &mut R) -> Result<SectionInfo, ParseError> {
    let mut buf = [0; SECT_HEADER_SIZE];
    f.read_exact(&mut buf[..]).map_err(clarify_err)?;

    let sect_size = read_as!(u32, buf, 0) as usize;
    let sect_num = buf[4];
    Ok(SectionInfo {
        num: sect_num,
        size: sect_size,
        body: None,
    })
}

// To convert
// io::Result<usize> (= Result<usize, io::Error) -> Result<usize, String>
fn clarify_err(e: io::Error) -> ParseError {
    let msg = format!("read error: {}", e.to_string());
    ParseError::ReadError(msg)
}
