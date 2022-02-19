use chrono::{offset::TimeZone, Utc};
use std::convert::TryInto;
use std::io::{self, Read, Seek, SeekFrom};
use std::result::Result;

use crate::codetables::SUPPORTED_PROD_DEF_TEMPLATE_NUMBERS;
use crate::context::{SectionBody, SectionInfo};
use crate::datatypes::*;
use crate::error::*;

const SECT0_IS_MAGIC: &[u8] = b"GRIB";
const SECT0_IS_MAGIC_SIZE: usize = SECT0_IS_MAGIC.len();
const SECT0_IS_SIZE: usize = 16;
const SECT_HEADER_SIZE: usize = 5;
const SECT8_ES_MAGIC: &[u8] = b"7777";
pub(crate) const SECT8_ES_SIZE: usize = SECT8_ES_MAGIC.len();

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

/// # Example
/// ```
/// use grib::context::{SectionBody, SectionInfo};
/// use grib::datatypes::Indicator;
/// use grib::reader::{Grib2SectionStream, SeekableGrib2Reader};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let f = std::fs::File::open(
///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
///     )?;
///     let f = std::io::BufReader::new(f);
///     let grib2_reader = SeekableGrib2Reader::new(f);
///
///     let mut sect_stream = Grib2SectionStream::new(grib2_reader);
///     assert_eq!(
///         sect_stream.next(),
///         Some(Ok(SectionInfo {
///             num: 0,
///             offset: 0,
///             size: 16,
///             body: Some(SectionBody::Section0(Indicator {
///                 discipline: 0,
///                 total_length: 193,
///             })),
///         }))
///     );
///     Ok(())
/// }
/// ```
pub struct Grib2SectionStream<R> {
    reader: R,
    whole_size: usize,
    rest_size: usize,
}

impl<R> Grib2SectionStream<R> {
    /// # Example
    /// ```
    /// use grib::reader::{Grib2SectionStream, SeekableGrib2Reader};
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
    ///     )?;
    ///     let mut f = std::io::BufReader::new(f);
    ///     let grib2_reader = SeekableGrib2Reader::new(f);
    ///     let _sect_stream = Grib2SectionStream::new(grib2_reader);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            whole_size: 0,
            rest_size: 0,
        }
    }

    pub fn into_reader(self) -> R {
        self.reader
    }
}

impl<R> Grib2SectionStream<R>
where
    R: Grib2Read,
{
    #[inline]
    fn next_sect0(&mut self) -> Option<Result<SectionInfo, ParseError>> {
        let offset = self.whole_size;
        let result = self.reader.read_sect0().transpose()?.map(|indicator| {
            let message_size = indicator.total_length as usize;
            self.whole_size += message_size;
            let sect_info = SectionInfo {
                num: 0,
                offset,
                size: SECT0_IS_SIZE,
                body: Some(SectionBody::Section0(indicator)),
            };
            self.rest_size = message_size - SECT0_IS_SIZE;
            sect_info
        });
        Some(result)
    }

    #[inline]
    fn next_sect8(&mut self) -> Option<Result<SectionInfo, ParseError>> {
        let result = self.reader.read_sect8().transpose()?.map(|_| {
            let sect_info = SectionInfo {
                num: 8,
                offset: self.whole_size - self.rest_size,
                size: SECT8_ES_SIZE,
                body: None,
            };
            self.rest_size -= SECT8_ES_SIZE;
            sect_info
        });
        Some(result)
    }

    #[inline]
    fn next_sect(&mut self) -> Option<Result<SectionInfo, ParseError>> {
        let result = self.reader.read_sect_header().transpose()?;
        match result {
            Ok(header) => {
                let offset = self.whole_size - self.rest_size;
                match self.reader.read_sect(&header) {
                    Ok(body) => {
                        let body = Some(body);
                        let (size, num) = header;
                        self.rest_size -= size;
                        Some(Ok(SectionInfo {
                            num,
                            offset,
                            size,
                            body,
                        }))
                    }
                    Err(e) => Some(Err(e)),
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl<R> Iterator for Grib2SectionStream<R>
where
    R: Grib2Read,
{
    type Item = Result<SectionInfo, ParseError>;

    fn next(&mut self) -> Option<Result<SectionInfo, ParseError>> {
        match self.rest_size {
            0 => self.next_sect0(),
            SECT8_ES_SIZE => self.next_sect8(),
            _ => self.next_sect(),
        }
    }
}

pub trait Grib2Read: Read + Seek {
    /// Reads Section 0.
    fn read_sect0(&mut self) -> Result<Option<Indicator>, ParseError>;

    /// Reads Section 8.
    fn read_sect8(&mut self) -> Result<Option<()>, ParseError>;

    /// Reads a common header for Sections 1-7 and returns the section
    /// size and number.
    fn read_sect_header(&mut self) -> Result<Option<SectHeader>, ParseError>;
    fn read_sect(&mut self, header: &SectHeader) -> Result<SectionBody, ParseError>;
    fn read_sect_body_bytes(&mut self, sect: &SectionInfo) -> Result<Box<[u8]>, ParseError>;
}

pub struct SeekableGrib2Reader<R> {
    reader: R,
}

impl<R> SeekableGrib2Reader<R> {
    pub fn new(r: R) -> Self {
        Self { reader: r }
    }
}

impl<R: Read> Read for SeekableGrib2Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.reader.read_exact(buf)
    }
}

impl<S: Seek> Seek for SeekableGrib2Reader<S> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.reader.seek(pos)
    }
}

macro_rules! check_size {
    ($size:expr, $expected_size:expr) => {{
        if $size == 0 {
            return Ok(None);
        }
        if $size != $expected_size {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "failed to fill whole buffer",
            )
            .into());
        }
    }};
}

impl<R: Read + Seek> Grib2Read for SeekableGrib2Reader<R> {
    fn read_sect0(&mut self) -> Result<Option<Indicator>, ParseError> {
        let mut buf = [0; SECT0_IS_SIZE];
        let size = self.read(&mut buf[..])?;
        check_size!(size, buf.len());

        if &buf[0..SECT0_IS_MAGIC_SIZE] != SECT0_IS_MAGIC {
            return Err(ParseError::NotGRIB);
        }
        let discipline = buf[6];
        let version = buf[7];
        if version != 2 {
            return Err(ParseError::GRIBVersionMismatch(version));
        }

        let fsize = read_as!(u64, buf, 8);

        Ok(Some(Indicator {
            discipline,
            total_length: fsize,
        }))
    }

    fn read_sect8(&mut self) -> Result<Option<()>, ParseError> {
        let mut buf = [0; SECT8_ES_SIZE];
        let size = self.read(&mut buf[..])?;
        check_size!(size, buf.len());

        if buf[..] != SECT8_ES_MAGIC[..] {
            return Err(ParseError::EndSectionMismatch);
        }

        Ok(Some(()))
    }

    fn read_sect_header(&mut self) -> Result<Option<SectHeader>, ParseError> {
        let mut buf = [0; SECT_HEADER_SIZE];
        let size = self.read(&mut buf[..])?;
        check_size!(size, buf.len());

        let sect_size = read_as!(u32, buf, 0) as usize;
        let sect_num = buf[4];

        Ok(Some((sect_size, sect_num)))
    }

    fn read_sect(&mut self, header: &SectHeader) -> Result<SectionBody, ParseError> {
        let (size, num) = header;
        let body_size = size - SECT_HEADER_SIZE;
        let body = match num {
            1 => unpack_sect1_body(self, body_size)?,
            2 => unpack_sect2_body(self, body_size)?,
            3 => unpack_sect3_body(self, body_size)?,
            4 => unpack_sect4_body(self, body_size)?,
            5 => unpack_sect5_body(self, body_size)?,
            6 => unpack_sect6_body(self, body_size)?,
            7 => skip_sect7_body(self, body_size)?,
            _ => return Err(ParseError::UnknownSectionNumber(*num)),
        };

        Ok(body)
    }

    fn read_sect_body_bytes(&mut self, sect: &SectionInfo) -> Result<Box<[u8]>, ParseError> {
        let body_offset = sect.offset + SECT_HEADER_SIZE;
        self.seek(SeekFrom::Start(body_offset as u64))?;

        let body_size = sect.size - SECT_HEADER_SIZE;
        let mut buf = vec![0; body_size];
        self.read_exact(buf.as_mut_slice())?;

        Ok(buf.into_boxed_slice())
    }
}

pub fn unpack_sect1_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 16]; // octet 6-21
    f.read_exact(&mut buf[..])?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..])?;
    }

    Ok(SectionBody::Section1(Identification {
        centre_id: read_as!(u16, buf, 0),
        subcentre_id: read_as!(u16, buf, 2),
        master_table_version: buf[4],
        local_table_version: buf[5],
        ref_time_significance: buf[6],
        ref_time: Utc
            .ymd(read_as!(u16, buf, 7).into(), buf[9].into(), buf[10].into())
            .and_hms(buf[11].into(), buf[12].into(), buf[13].into()),
        prod_status: buf[14],
        data_type: buf[15],
    }))
}

pub fn unpack_sect2_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let len_extra = body_size;
    if len_extra > 0 {
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..])?;
    }

    Ok(SectionBody::Section2)
}

pub fn unpack_sect3_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 9]; // octet 6-14
    f.read_exact(&mut buf[..])?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..])?;
    }

    Ok(SectionBody::Section3(GridDefinition {
        num_points: read_as!(u32, buf, 1),
        grid_tmpl_num: read_as!(u16, buf, 7),
    }))
}

pub fn unpack_sect4_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 4]; // octet 6-9
    f.read_exact(&mut buf[..])?;

    let len_extra = body_size - buf.len();
    let mut templated = vec![0; len_extra];
    f.read_exact(&mut templated[..])?;

    let prod_tmpl_num = read_as!(u16, buf, 2);

    Ok(SectionBody::Section4(ProdDefinition {
        num_coordinates: read_as!(u16, buf, 0),
        prod_tmpl_num,
        templated: templated.into_boxed_slice(),
        template_supported: SUPPORTED_PROD_DEF_TEMPLATE_NUMBERS.contains(&prod_tmpl_num),
    }))
}

pub fn unpack_sect5_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 6]; // octet 6-11
    f.read_exact(&mut buf[..])?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..])?;
    }

    Ok(SectionBody::Section5(ReprDefinition {
        num_points: read_as!(u32, buf, 0),
        repr_tmpl_num: read_as!(u16, buf, 4),
    }))
}

pub fn unpack_sect6_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let mut buf = [0; 1]; // octet 6
    f.read_exact(&mut buf[..])?;

    let len_extra = body_size - buf.len();
    if len_extra > 0 {
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..])?;
    }

    Ok(SectionBody::Section6(BitMap {
        bitmap_indicator: buf[0],
    }))
}

fn skip_sect7_body<R: Seek>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    f.seek(SeekFrom::Current(body_size as i64))?;

    Ok(SectionBody::Section7)
}

type SectHeader = (usize, u8);

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn read_one_grib2_message() -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )?;
        let f = std::io::BufReader::new(f);

        let grib2_reader = SeekableGrib2Reader::new(f);
        let sect_stream = Grib2SectionStream::new(grib2_reader);
        assert_eq!(
            sect_stream
                .take(10)
                .map(|result| result.map(|sect| (sect.num, sect.offset, sect.size)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 16)),
                Ok((1, 16, 21)),
                Ok((2, 37, 27)),
                Ok((3, 64, 35)),
                Ok((4, 99, 58)),
                Ok((5, 157, 21)),
                Ok((6, 178, 6)),
                Ok((7, 184, 5)),
                Ok((8, 189, 4)),
            ]
        );

        Ok(())
    }

    #[test]
    fn read_multiple_grib2_messages() -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )?;
        let mut f = std::io::BufReader::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let repeated_message = buf.repeat(2);
        let f = Cursor::new(repeated_message);

        let grib2_reader = SeekableGrib2Reader::new(f);
        let sect_stream = Grib2SectionStream::new(grib2_reader);
        assert_eq!(
            sect_stream
                .take(19)
                .map(|result| result.map(|sect| (sect.num, sect.offset, sect.size)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 16)),
                Ok((1, 16, 21)),
                Ok((2, 37, 27)),
                Ok((3, 64, 35)),
                Ok((4, 99, 58)),
                Ok((5, 157, 21)),
                Ok((6, 178, 6)),
                Ok((7, 184, 5)),
                Ok((8, 189, 4)),
                Ok((0, 193, 16)),
                Ok((1, 209, 21)),
                Ok((2, 230, 27)),
                Ok((3, 257, 35)),
                Ok((4, 292, 58)),
                Ok((5, 350, 21)),
                Ok((6, 371, 6)),
                Ok((7, 377, 5)),
                Ok((8, 382, 4)),
            ]
        );

        Ok(())
    }

    #[test]
    fn read_grib2_message_with_incomplete_section_0() -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )?;
        let mut f = std::io::BufReader::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let mut extra_bytes = "extra".as_bytes().to_vec();
        buf.append(&mut extra_bytes);
        let f = Cursor::new(buf);

        let grib2_reader = SeekableGrib2Reader::new(f);
        let sect_stream = Grib2SectionStream::new(grib2_reader);
        assert_eq!(
            sect_stream
                .take(10)
                .map(|result| result.map(|sect| (sect.num, sect.offset, sect.size)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 16)),
                Ok((1, 16, 21)),
                Ok((2, 37, 27)),
                Ok((3, 64, 35)),
                Ok((4, 99, 58)),
                Ok((5, 157, 21)),
                Ok((6, 178, 6)),
                Ok((7, 184, 5)),
                Ok((8, 189, 4)),
                Err(ParseError::ReadError(
                    "failed to fill whole buffer".to_owned()
                ))
            ]
        );

        Ok(())
    }

    #[test]
    fn read_grib2_message_with_incomplete_section_1() -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )?;
        let mut f = std::io::BufReader::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let mut message_2_bytes = buf[..(SECT0_IS_SIZE + 1)].to_vec();
        buf.append(&mut message_2_bytes);
        let f = Cursor::new(buf);

        let grib2_reader = SeekableGrib2Reader::new(f);
        let sect_stream = Grib2SectionStream::new(grib2_reader);
        assert_eq!(
            sect_stream
                .take(19)
                .map(|result| result.map(|sect| (sect.num, sect.offset, sect.size)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 16)),
                Ok((1, 16, 21)),
                Ok((2, 37, 27)),
                Ok((3, 64, 35)),
                Ok((4, 99, 58)),
                Ok((5, 157, 21)),
                Ok((6, 178, 6)),
                Ok((7, 184, 5)),
                Ok((8, 189, 4)),
                Ok((0, 193, 16)),
                Err(ParseError::ReadError(
                    "failed to fill whole buffer".to_owned()
                ))
            ]
        );

        Ok(())
    }

    #[test]
    fn read_grib2_message_with_incomplete_section_8() -> Result<(), Box<dyn std::error::Error>> {
        let f = std::fs::File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )?;
        let mut f = std::io::BufReader::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let mut repeated_message = buf.repeat(2);
        repeated_message.pop();
        let f = Cursor::new(repeated_message);

        let grib2_reader = SeekableGrib2Reader::new(f);
        let sect_stream = Grib2SectionStream::new(grib2_reader);
        assert_eq!(
            sect_stream
                .take(19)
                .map(|result| result.map(|sect| (sect.num, sect.offset, sect.size)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 16)),
                Ok((1, 16, 21)),
                Ok((2, 37, 27)),
                Ok((3, 64, 35)),
                Ok((4, 99, 58)),
                Ok((5, 157, 21)),
                Ok((6, 178, 6)),
                Ok((7, 184, 5)),
                Ok((8, 189, 4)),
                Ok((0, 193, 16)),
                Ok((1, 209, 21)),
                Ok((2, 230, 27)),
                Ok((3, 257, 35)),
                Ok((4, 292, 58)),
                Ok((5, 350, 21)),
                Ok((6, 371, 6)),
                Ok((7, 377, 5)),
                Err(ParseError::ReadError(
                    "failed to fill whole buffer".to_owned()
                ))
            ]
        );

        Ok(())
    }
}
