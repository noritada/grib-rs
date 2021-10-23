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
const SECT8_ES_SIZE: usize = SECT8_ES_MAGIC.len();

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

pub trait Grib2Read: Read + Seek {
    fn scan(&mut self) -> Result<Box<[SectionInfo]>, ParseError> {
        let indicator = self.read_sect0()?;
        let whole_size = indicator.total_length as usize;
        let mut rest_size = whole_size - SECT0_IS_SIZE;
        let mut sects = vec![SectionInfo {
            num: 0,
            offset: 0,
            size: SECT0_IS_SIZE,
            body: Some(SectionBody::Section0(indicator)),
        }];

        loop {
            if rest_size == SECT8_ES_SIZE {
                self.read_sect8()?;
                let sect_info = SectionInfo {
                    num: 8,
                    offset: whole_size - rest_size,
                    size: SECT8_ES_SIZE,
                    body: None,
                };
                sects.push(sect_info);
                break;
            }

            let mut sect_info = self.read_sect_meta()?;
            sect_info.offset = whole_size - rest_size;
            sect_info.body = Some(self.read_sect(&sect_info)?);
            rest_size -= sect_info.size;
            sects.push(sect_info);
        }

        Ok(sects.into_boxed_slice())
    }

    fn read_sect0(&mut self) -> Result<Indicator, ParseError>;
    fn read_sect8(&mut self) -> Result<(), ParseError>;
    fn read_sect_meta(&mut self) -> Result<SectionInfo, ParseError>;
    fn read_sect(&mut self, meta: &SectionInfo) -> Result<SectionBody, ParseError>;
    fn read_sect_body_bytes(&mut self, meta: &SectionInfo) -> Result<Box<[u8]>, ParseError>;
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

impl<R: Read + Seek> Grib2Read for SeekableGrib2Reader<R> {
    fn read_sect0(&mut self) -> Result<Indicator, ParseError> {
        let mut buf = [0; SECT0_IS_SIZE];
        self.read_exact(&mut buf[..])
            .map_err(|e| ParseError::FileTypeCheckError(e.to_string()))?;

        if &buf[0..SECT0_IS_MAGIC_SIZE] != SECT0_IS_MAGIC {
            return Err(ParseError::NotGRIB);
        }
        let discipline = buf[6];
        let version = buf[7];
        if version != 2 {
            return Err(ParseError::GRIBVersionMismatch(version));
        }

        let fsize = read_as!(u64, buf, 8);

        Ok(Indicator {
            discipline,
            total_length: fsize,
        })
    }

    fn read_sect8(&mut self) -> Result<(), ParseError> {
        let mut buf = [0; SECT8_ES_SIZE];
        self.read_exact(&mut buf[..])?;

        if buf[..] != SECT8_ES_MAGIC[..] {
            return Err(ParseError::EndSectionMismatch);
        }

        Ok(())
    }

    /// Reads a common header for sections 1-7 and returns the section
    /// number and size.  Since offset is not determined within this
    /// function, the `offset` and `body` fields in returned `SectionInfo`
    /// struct is set to `0` and `None` respectively.
    fn read_sect_meta(&mut self) -> Result<SectionInfo, ParseError> {
        let mut buf = [0; SECT_HEADER_SIZE];
        self.read_exact(&mut buf[..])?;

        let sect_size = read_as!(u32, buf, 0) as usize;
        let sect_num = buf[4];

        Ok(SectionInfo {
            num: sect_num,
            offset: 0,
            size: sect_size,
            body: None,
        })
    }

    fn read_sect(&mut self, meta: &SectionInfo) -> Result<SectionBody, ParseError> {
        let body_size = meta.size - SECT_HEADER_SIZE;
        let body = match meta.num {
            1 => unpack_sect1_body(self, body_size)?,
            2 => unpack_sect2_body(self, body_size)?,
            3 => unpack_sect3_body(self, body_size)?,
            4 => unpack_sect4_body(self, body_size)?,
            5 => unpack_sect5_body(self, body_size)?,
            6 => unpack_sect6_body(self, body_size)?,
            7 => skip_sect7_body(self, body_size)?,
            _ => return Err(ParseError::UnknownSectionNumber(meta.num)),
        };

        Ok(body)
    }

    fn read_sect_body_bytes(&mut self, meta: &SectionInfo) -> Result<Box<[u8]>, ParseError> {
        let body_offset = meta.offset + SECT_HEADER_SIZE;
        self.seek(SeekFrom::Start(body_offset as u64))?;

        let body_size = meta.size - SECT_HEADER_SIZE;
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

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::{BufReader, Cursor};
    use xz2::bufread::XzDecoder;

    #[test]
    fn read_normal() -> Result<(), Box<dyn std::error::Error>> {
        let f = File::open(
            "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz",
        )
        .unwrap();
        let f = BufReader::new(f);
        let mut f = XzDecoder::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let f = Cursor::new(buf);

        assert_eq!(
            SeekableGrib2Reader::new(f).scan(),
            Ok(vec![
                SectionInfo {
                    num: 0,
                    offset: 0,
                    size: 16,
                    body: Some(SectionBody::Section0(Indicator {
                        discipline: 0,
                        total_length: 10321,
                    })),
                },
                SectionInfo {
                    num: 1,
                    offset: 16,
                    size: 21,
                    body: Some(SectionBody::Section1(Identification {
                        centre_id: 34,
                        subcentre_id: 0,
                        master_table_version: 5,
                        local_table_version: 1,
                        ref_time_significance: 0,
                        ref_time: Utc.ymd(2016, 8, 22).and_hms(2, 0, 0),
                        prod_status: 0,
                        data_type: 2,
                    })),
                },
                SectionInfo {
                    num: 3,
                    offset: 37,
                    size: 72,
                    body: Some(SectionBody::Section3(GridDefinition {
                        num_points: 86016,
                        grid_tmpl_num: 0,
                    })),
                },
                SectionInfo {
                    num: 4,
                    offset: 109,
                    size: 34,
                    body: Some(SectionBody::Section4(ProdDefinition {
                        num_coordinates: 0,
                        prod_tmpl_num: 0,
                        templated: vec![
                            193, 0, 0, 153, 255, 0, 0, 0, 0, 0, 0, 0, 0, 1, 255, 255, 255, 255,
                            255, 255, 255, 255, 255, 255, 255,
                        ]
                        .into_boxed_slice(),
                        template_supported: true,
                    })),
                },
                SectionInfo {
                    num: 5,
                    offset: 143,
                    size: 23,
                    body: Some(SectionBody::Section5(ReprDefinition {
                        num_points: 86016,
                        repr_tmpl_num: 200,
                    })),
                },
                SectionInfo {
                    num: 6,
                    offset: 166,
                    size: 6,
                    body: Some(SectionBody::Section6(BitMap {
                        bitmap_indicator: 255,
                    })),
                },
                SectionInfo {
                    num: 7,
                    offset: 172,
                    size: 1391,
                    body: Some(SectionBody::Section7),
                },
                SectionInfo {
                    num: 4,
                    offset: 1563,
                    size: 34,
                    body: Some(SectionBody::Section4(ProdDefinition {
                        num_coordinates: 0,
                        prod_tmpl_num: 0,
                        templated: vec![
                            193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 10, 1, 255, 255, 255, 255,
                            255, 255, 255, 255, 255, 255, 255,
                        ]
                        .into_boxed_slice(),
                        template_supported: true,
                    })),
                },
                SectionInfo {
                    num: 5,
                    offset: 1597,
                    size: 23,
                    body: Some(SectionBody::Section5(ReprDefinition {
                        num_points: 86016,
                        repr_tmpl_num: 200,
                    })),
                },
                SectionInfo {
                    num: 6,
                    offset: 1620,
                    size: 6,
                    body: Some(SectionBody::Section6(BitMap {
                        bitmap_indicator: 255,
                    })),
                },
                SectionInfo {
                    num: 7,
                    offset: 1626,
                    size: 1399,
                    body: Some(SectionBody::Section7),
                },
                SectionInfo {
                    num: 4,
                    offset: 3025,
                    size: 34,
                    body: Some(SectionBody::Section4(ProdDefinition {
                        num_coordinates: 0,
                        prod_tmpl_num: 0,
                        templated: vec![
                            193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 20, 1, 255, 255, 255, 255,
                            255, 255, 255, 255, 255, 255, 255
                        ]
                        .into_boxed_slice(),
                        template_supported: true,
                    })),
                },
                SectionInfo {
                    num: 5,
                    offset: 3059,
                    size: 23,
                    body: Some(SectionBody::Section5(ReprDefinition {
                        num_points: 86016,
                        repr_tmpl_num: 200,
                    })),
                },
                SectionInfo {
                    num: 6,
                    offset: 3082,
                    size: 6,
                    body: Some(SectionBody::Section6(BitMap {
                        bitmap_indicator: 255,
                    })),
                },
                SectionInfo {
                    num: 7,
                    offset: 3088,
                    size: 1404,
                    body: Some(SectionBody::Section7),
                },
                SectionInfo {
                    num: 4,
                    offset: 4492,
                    size: 34,
                    body: Some(SectionBody::Section4(ProdDefinition {
                        num_coordinates: 0,
                        prod_tmpl_num: 0,
                        templated: vec![
                            193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 30, 1, 255, 255, 255, 255,
                            255, 255, 255, 255, 255, 255, 255
                        ]
                        .into_boxed_slice(),
                        template_supported: true,
                    })),
                },
                SectionInfo {
                    num: 5,
                    offset: 4526,
                    size: 23,
                    body: Some(SectionBody::Section5(ReprDefinition {
                        num_points: 86016,
                        repr_tmpl_num: 200,
                    })),
                },
                SectionInfo {
                    num: 6,
                    offset: 4549,
                    size: 6,
                    body: Some(SectionBody::Section6(BitMap {
                        bitmap_indicator: 255,
                    })),
                },
                SectionInfo {
                    num: 7,
                    offset: 4555,
                    size: 1395,
                    body: Some(SectionBody::Section7),
                },
                SectionInfo {
                    num: 4,
                    offset: 5950,
                    size: 34,
                    body: Some(SectionBody::Section4(ProdDefinition {
                        num_coordinates: 0,
                        prod_tmpl_num: 0,
                        templated: vec![
                            193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 40, 1, 255, 255, 255, 255,
                            255, 255, 255, 255, 255, 255, 255
                        ]
                        .into_boxed_slice(),
                        template_supported: true,
                    })),
                },
                SectionInfo {
                    num: 5,
                    offset: 5984,
                    size: 23,
                    body: Some(SectionBody::Section5(ReprDefinition {
                        num_points: 86016,
                        repr_tmpl_num: 200,
                    })),
                },
                SectionInfo {
                    num: 6,
                    offset: 6007,
                    size: 6,
                    body: Some(SectionBody::Section6(BitMap {
                        bitmap_indicator: 255,
                    })),
                },
                SectionInfo {
                    num: 7,
                    offset: 6013,
                    size: 1395,
                    body: Some(SectionBody::Section7),
                },
                SectionInfo {
                    num: 4,
                    offset: 7408,
                    size: 34,
                    body: Some(SectionBody::Section4(ProdDefinition {
                        num_coordinates: 0,
                        prod_tmpl_num: 0,
                        templated: vec![
                            193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 50, 1, 255, 255, 255, 255,
                            255, 255, 255, 255, 255, 255, 255
                        ]
                        .into_boxed_slice(),
                        template_supported: true,
                    })),
                },
                SectionInfo {
                    num: 5,
                    offset: 7442,
                    size: 23,
                    body: Some(SectionBody::Section5(ReprDefinition {
                        num_points: 86016,
                        repr_tmpl_num: 200,
                    })),
                },
                SectionInfo {
                    num: 6,
                    offset: 7465,
                    size: 6,
                    body: Some(SectionBody::Section6(BitMap {
                        bitmap_indicator: 255,
                    })),
                },
                SectionInfo {
                    num: 7,
                    offset: 7471,
                    size: 1397,
                    body: Some(SectionBody::Section7),
                },
                SectionInfo {
                    num: 4,
                    offset: 8868,
                    size: 34,
                    body: Some(SectionBody::Section4(ProdDefinition {
                        num_coordinates: 0,
                        prod_tmpl_num: 0,
                        templated: vec![
                            193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 60, 1, 255, 255, 255, 255,
                            255, 255, 255, 255, 255, 255, 255
                        ]
                        .into_boxed_slice(),
                        template_supported: true,
                    })),
                },
                SectionInfo {
                    num: 5,
                    offset: 8902,
                    size: 23,
                    body: Some(SectionBody::Section5(ReprDefinition {
                        num_points: 86016,
                        repr_tmpl_num: 200,
                    })),
                },
                SectionInfo {
                    num: 6,
                    offset: 8925,
                    size: 6,
                    body: Some(SectionBody::Section6(BitMap {
                        bitmap_indicator: 255,
                    })),
                },
                SectionInfo {
                    num: 7,
                    offset: 8931,
                    size: 1386,
                    body: Some(SectionBody::Section7),
                },
                SectionInfo {
                    num: 8,
                    offset: 10317,
                    size: 4,
                    body: None
                },
            ]
            .into_boxed_slice())
        );

        Ok(())
    }
}
