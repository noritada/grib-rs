use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read, Seek, SeekFrom};
use std::result::Result;

use crate::codetables::{
    lookup_table, CODE_TABLE_1_0, CODE_TABLE_1_1, CODE_TABLE_1_2, CODE_TABLE_1_3, CODE_TABLE_1_4,
};

const SECT0_IS_MAGIC: &'static [u8] = b"GRIB";
const SECT0_IS_MAGIC_SIZE: usize = SECT0_IS_MAGIC.len();
const SECT0_IS_SIZE: usize = 16;
const SECT_HEADER_SIZE: usize = 5;
const SECT8_ES_MAGIC: &'static [u8] = b"7777";
const SECT8_ES_SIZE: usize = SECT8_ES_MAGIC.len();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SectionInfo {
    pub num: u8,
    pub offset: usize,
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

    pub fn skip_body<S: Seek>(&self, f: &mut S) -> Result<(), ParseError> {
        let body_size = self.size - SECT_HEADER_SIZE;
        f.seek(SeekFrom::Current(body_size as i64))?; // < std::io::Seek
        Ok(())
    }

    pub fn get_tmpl_code(&self) -> Option<TemplateInfo> {
        let tmpl_num = self.body.as_ref()?.get_tmpl_num()?;
        Some(TemplateInfo(self.num, tmpl_num))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SectionBody {
    Section1(Identification),
    Section2,
    Section3(GridDefinition),
    Section4(ProdDefinition),
    Section5(ReprDefinition),
    Section6 {
        /// Bit-map indicator
        bitmap_indicator: u8,
    },
    Section7,
}

impl SectionBody {
    fn get_tmpl_num(&self) -> Option<u16> {
        match self {
            Self::Section3(s) => Some(s.grid_tmpl_num),
            Self::Section4(s) => Some(s.prod_tmpl_num),
            Self::Section5(s) => Some(s.repr_tmpl_num),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identification {
    /// Identification of originating/generating centre (see Common Code Table C-1)
    centre_id: u16,
    /// Identification of originating/generating sub-centre (allocated by originating/ generating centre)
    subcentre_id: u16,
    /// GRIB Master Tables Version Number (see Code Table 1.0)
    master_table_version: u8,
    /// GRIB Local Tables Version Number (see Code Table 1.1)
    local_table_version: u8,
    /// Significance of Reference Time (see Code Table 1.2)
    ref_time_significance: u8,
    /// Reference time of data
    ref_time: RefTime,
    /// Production status of processed data in this GRIB message
    /// (see Code Table 1.3)
    prod_status: u8,
    /// Type of processed data in this GRIB message (see Code Table 1.4)
    data_type: u8,
}

impl Display for Identification {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "\
Originating/generating centre:          {}
Originating/generating sub-centre:      {}
GRIB Master Tables Version Number:      {}
GRIB Local Tables Version Number:       {}
Significance of Reference Time:         {}
Reference time of data:                 {}
Production status of processed data:    {}
Type of processed data:                 {}\
",
            self.centre_id,
            self.subcentre_id,
            lookup_table(CODE_TABLE_1_0, self.master_table_version),
            lookup_table(CODE_TABLE_1_1, self.local_table_version),
            lookup_table(CODE_TABLE_1_2, self.ref_time_significance),
            self.ref_time,
            lookup_table(CODE_TABLE_1_3, self.prod_status),
            lookup_table(CODE_TABLE_1_4, self.data_type)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GridDefinition {
    /// Number of data points
    num_points: u32,
    /// Grid Definition Template Number
    grid_tmpl_num: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProdDefinition {
    /// Number of coordinate values after Template
    num_coordinates: u16,
    /// Product Definition Template Number
    prod_tmpl_num: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReprDefinition {
    /// Number of data points where one or more values are
    /// specified in Section 7 when a bit map is present, total
    /// number of data points when a bit map is absent
    num_points: u32,
    /// Data Representation Template Number
    repr_tmpl_num: u16,
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

impl Display for RefTime {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}Z",
            self.year, self.month, self.date, self.hour, self.minute, self.second
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubMessage {
    section2: Option<usize>,
    section3: Option<usize>,
    section4: Option<usize>,
    section5: Option<usize>,
    section6: Option<usize>,
    section7: Option<usize>,
}

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TemplateInfo(pub u8, pub u16);

impl Display for TemplateInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

pub trait GribReader<R: Read> {
    fn new(f: R) -> Result<Self, ParseError>
    where
        Self: Sized;
}

pub struct Grib2FileReader<R: Read> {
    reader: R,
    sections: Box<[SectionInfo]>,
    submessages: Box<[SubMessage]>,
}

impl<R: Read> Grib2FileReader<R> {
    pub fn submessages(&self) -> &Box<[SubMessage]> {
        &self.submessages
    }

    pub fn list_templates(&self) -> Vec<TemplateInfo> {
        get_templates(&self.sections)
    }
}

impl<R: Read> GribReader<R> for Grib2FileReader<R> {
    fn new(mut f: R) -> Result<Self, ParseError>
    where
        Self: Sized,
    {
        let sects = scan(&mut f)?;
        let submessages = get_submessages(&sects)?;
        Ok(Self {
            reader: f,
            sections: sects,
            submessages: submessages,
        })
    }
}

impl<R: Read> Display for Grib2FileReader<R> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let err = "No information available".to_string();
        let s = match self.sections.first() {
            Some(SectionInfo {
                body: Some(SectionBody::Section1(body)),
                ..
            }) => format!("{}", body),
            _ => err,
        };
        write!(f, "{}", s)
    }
}

fn scan<R: Read>(mut f: R) -> Result<Box<[SectionInfo]>, ParseError> {
    let whole_size = unpack_sect0(&mut f)?;
    let mut rest_size = whole_size - SECT0_IS_SIZE;
    let mut sects = Vec::new();

    loop {
        if rest_size == SECT8_ES_SIZE {
            unpack_sect8(&mut f)?;
            let sect_info = SectionInfo {
                num: 8,
                offset: whole_size - rest_size,
                size: SECT8_ES_SIZE,
                body: None,
            };
            sects.push(sect_info);
            break;
        }

        let mut sect_info = unpack_sect_header(&mut f)?;
        sect_info.offset = whole_size - rest_size;
        // Some readers such as flate2::gz::read::GzDecoder do not
        // implement Seek.
        // let _sect_body = sect_info.skip_body(&mut f)?;
        sect_info.body = Some(sect_info.read_body(&mut f)?);
        rest_size -= sect_info.size;
        sects.push(sect_info);
    }

    Ok(sects.into_boxed_slice())
}

/// Validates the section order of sections and split them into a
/// vector of section groups.
fn get_submessages(sects: &Box<[SectionInfo]>) -> Result<Box<[SubMessage]>, ParseError> {
    let mut iter = sects.iter().enumerate();
    let mut starts = Vec::new();
    let mut i2_default = None;
    let mut i3_default = None;

    macro_rules! check {
        ($num:expr) => {{
            let (i, sect) = iter
                .next()
                .ok_or(ParseError::GRIB2IterationSuddenlyFinished)?;
            if sect.num != $num {
                return Err(ParseError::GRIB2WrongIteration(i));
            }
            i
        }};
    }

    macro_rules! update_default {
        ($submessage:expr) => {{
            let submessage = $submessage;
            i2_default = submessage.section2;
            i3_default = submessage.section3;
            submessage
        }};
    }

    check!(1);

    loop {
        let sect = iter.next();
        let start = match sect {
            Some((_i, SectionInfo { num: 2, .. })) => {
                let (i, _) = sect.unwrap();
                let i3 = check!(3);
                let i4 = check!(4);
                let i5 = check!(5);
                let i6 = check!(6);
                let i7 = check!(7);
                update_default!(SubMessage {
                    section2: Some(i),
                    section3: Some(i3),
                    section4: Some(i4),
                    section5: Some(i5),
                    section6: Some(i6),
                    section7: Some(i7),
                })
            }
            Some((_i, SectionInfo { num: 3, .. })) => {
                let (i, _) = sect.unwrap();
                let i4 = check!(4);
                let i5 = check!(5);
                let i6 = check!(6);
                let i7 = check!(7);
                update_default!(SubMessage {
                    section2: i2_default,
                    section3: Some(i),
                    section4: Some(i4),
                    section5: Some(i5),
                    section6: Some(i6),
                    section7: Some(i7),
                })
            }
            Some((i, SectionInfo { num: 4, .. })) => {
                if i3_default == None {
                    return Err(ParseError::NoGridDefinition(i));
                }
                let (i, _) = sect.unwrap();
                let i5 = check!(5);
                let i6 = check!(6);
                let i7 = check!(7);
                update_default!(SubMessage {
                    section2: i2_default,
                    section3: i3_default,
                    section4: Some(i),
                    section5: Some(i5),
                    section6: Some(i6),
                    section7: Some(i7),
                })
            }
            Some((i, SectionInfo { num: 8, .. })) => {
                if i3_default == None {
                    return Err(ParseError::NoGridDefinition(i));
                }
                if i < sects.len() - 1 {
                    return Err(ParseError::GRIB2WrongIteration(i));
                }
                break;
            }
            Some((i, SectionInfo { .. })) => {
                return Err(ParseError::GRIB2WrongIteration(i));
            }
            None => {
                return Err(ParseError::GRIB2IterationSuddenlyFinished);
            }
        };
        starts.push(start);
    }

    Ok(starts.into_boxed_slice())
}

fn get_templates(sects: &Box<[SectionInfo]>) -> Vec<TemplateInfo> {
    let uniq: HashSet<_> = sects.iter().filter_map(|s| s.get_tmpl_code()).collect();
    let mut vec: Vec<_> = uniq.into_iter().collect();
    vec.sort_unstable();
    vec
}

pub fn unpack_sect0<R: Read>(f: &mut R) -> Result<usize, ParseError> {
    let mut buf = [0; SECT0_IS_SIZE];
    f.read_exact(&mut buf[..])?;

    if &buf[0..SECT0_IS_MAGIC_SIZE] != SECT0_IS_MAGIC {
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
        ref_time: RefTime {
            year: read_as!(u16, buf, 7),
            month: buf[9],
            date: buf[10],
            hour: buf[11],
            minute: buf[12],
            second: buf[13],
        },
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
    if len_extra > 0 {
        let mut buf = vec![0; len_extra];
        f.read_exact(&mut buf[..])?;
    }

    Ok(SectionBody::Section4(ProdDefinition {
        num_coordinates: read_as!(u16, buf, 0),
        prod_tmpl_num: read_as!(u16, buf, 2),
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

    Ok(SectionBody::Section6 {
        bitmap_indicator: buf[0],
    })
}

pub fn unpack_sect7_body<R: Read>(f: &mut R, body_size: usize) -> Result<SectionBody, ParseError> {
    let len_extra = body_size;
    if len_extra > 0 {
        let mut buf = vec![0; len_extra]; // octet 6-21
        f.read_exact(&mut buf[..])?;
    }

    Ok(SectionBody::Section7)
}

pub fn unpack_sect8<R: Read>(f: &mut R) -> Result<(), ParseError> {
    let mut buf = [0; SECT8_ES_SIZE];
    f.read_exact(&mut buf[..])?;

    if buf[..] != SECT8_ES_MAGIC[..] {
        return Err(ParseError::EndSectionMismatch);
    }

    Ok(())
}

/// Reads a common header for sections 1-7 and returns the section
/// number and size.  Since offset is not determined within this
/// function, the `offset` and `body` fields in returned `SectionInfo`
/// struct is set to `0` and `None` respectively.
pub fn unpack_sect_header<R: Read>(f: &mut R) -> Result<SectionInfo, ParseError> {
    let mut buf = [0; SECT_HEADER_SIZE];
    f.read_exact(&mut buf[..])?;

    let sect_size = read_as!(u32, buf, 0) as usize;
    let sect_num = buf[4];
    Ok(SectionInfo {
        num: sect_num,
        offset: 0,
        size: sect_size,
        body: None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParseError {
    ReadError(String),
    NotGRIB,
    GRIBVersionMismatch(u8),
    UnknownSectionNumber(u8),
    EndSectionMismatch,
    GRIB2IterationSuddenlyFinished,
    NoGridDefinition(usize),
    GRIB2WrongIteration(usize),
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        Self::ReadError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::BufReader;
    use xz2::bufread::XzDecoder;

    macro_rules! sect_placeholder {
        ($num:expr) => {{
            SectionInfo {
                num: $num,
                offset: 0,
                size: 0,
                body: None,
            }
        }};
    }

    macro_rules! sect_list {
        ($($num:expr,)*) => {{
            vec![
                $(
                    SectionInfo { num: $num, offset: 0, size: 0, body: None },
                )*
            ].into_boxed_slice()
        }}
    }

    #[test]
    fn read_normal() {
        let f = File::open(
            "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz",
        )
        .unwrap();
        let f = BufReader::new(f);
        let f = XzDecoder::new(f);

        assert_eq!(
            scan(f),
            Ok(vec![
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
                        ref_time: RefTime {
                            year: 2016,
                            month: 8,
                            date: 22,
                            hour: 2,
                            minute: 0,
                            second: 0,
                        },
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
                    body: Some(SectionBody::Section6 {
                        bitmap_indicator: 255,
                    }),
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
                    body: Some(SectionBody::Section6 {
                        bitmap_indicator: 255,
                    }),
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
                    body: Some(SectionBody::Section6 {
                        bitmap_indicator: 255,
                    }),
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
                    body: Some(SectionBody::Section6 {
                        bitmap_indicator: 255,
                    }),
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
                    body: Some(SectionBody::Section6 {
                        bitmap_indicator: 255,
                    }),
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
                    body: Some(SectionBody::Section6 {
                        bitmap_indicator: 255,
                    }),
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
                    body: Some(SectionBody::Section6 {
                        bitmap_indicator: 255,
                    }),
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
    }

    #[test]
    fn get_submessages_simple() {
        let sects = sect_list![1, 2, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![SubMessage {
                section2: Some(1),
                section3: Some(2),
                section4: Some(3),
                section5: Some(4),
                section6: Some(5),
                section7: Some(6),
            },]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_sect2_loop() {
        let sects = sect_list![1, 2, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: Some(1),
                    section3: Some(2),
                    section4: Some(3),
                    section5: Some(4),
                    section6: Some(5),
                    section7: Some(6),
                },
                SubMessage {
                    section2: Some(7),
                    section3: Some(8),
                    section4: Some(9),
                    section5: Some(10),
                    section6: Some(11),
                    section7: Some(12),
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_sect3_loop() {
        let sects = sect_list![1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: Some(1),
                    section3: Some(2),
                    section4: Some(3),
                    section5: Some(4),
                    section6: Some(5),
                    section7: Some(6),
                },
                SubMessage {
                    section2: Some(1),
                    section3: Some(7),
                    section4: Some(8),
                    section5: Some(9),
                    section6: Some(10),
                    section7: Some(11),
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_sect3_loop_no_sect2() {
        let sects = sect_list![1, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: None,
                    section3: Some(1),
                    section4: Some(2),
                    section5: Some(3),
                    section6: Some(4),
                    section7: Some(5),
                },
                SubMessage {
                    section2: None,
                    section3: Some(6),
                    section4: Some(7),
                    section5: Some(8),
                    section6: Some(9),
                    section7: Some(10),
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_sect4_loop() {
        let sects = sect_list![1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: Some(1),
                    section3: Some(2),
                    section4: Some(3),
                    section5: Some(4),
                    section6: Some(5),
                    section7: Some(6),
                },
                SubMessage {
                    section2: Some(1),
                    section3: Some(2),
                    section4: Some(7),
                    section5: Some(8),
                    section6: Some(9),
                    section7: Some(10),
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_sect4_loop_no_sect2() {
        let sects = sect_list![1, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: None,
                    section3: Some(1),
                    section4: Some(2),
                    section5: Some(3),
                    section6: Some(4),
                    section7: Some(5),
                },
                SubMessage {
                    section2: None,
                    section3: Some(1),
                    section4: Some(6),
                    section5: Some(7),
                    section6: Some(8),
                    section7: Some(9),
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_end_after_sect1() {
        let sects = sect_list![1,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect2_loop_1() {
        let sects = sect_list![1, 2,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect2_loop_2() {
        let sects = sect_list![1, 2, 3,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect3_loop_1() {
        let sects = sect_list![1, 3,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect3_loop_2() {
        let sects = sect_list![1, 3, 4,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect4_loop_1() {
        let sects = sect_list![1, 2, 3, 4, 5, 6, 7, 4,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect4_loop_2() {
        let sects = sect_list![1, 2, 3, 4, 5, 6, 7, 4, 5,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_no_grid_in_sect4() {
        let sects = sect_list![1, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::NoGridDefinition(1))
        );
    }

    #[test]
    fn get_submessages_no_grid_in_sect8() {
        let sects = sect_list![1, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::NoGridDefinition(1))
        );
    }

    #[test]
    fn get_submessages_wrong_order_in_sect2() {
        let sects = sect_list![1, 2, 4, 3, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2WrongIteration(2))
        );
    }

    #[test]
    fn get_submessages_wrong_order_in_sect3() {
        let sects = sect_list![1, 3, 5, 4, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2WrongIteration(2))
        );
    }

    #[test]
    fn get_submessages_wrong_order_in_sect4() {
        let sects = sect_list![1, 3, 4, 5, 6, 7, 4, 6, 5, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ParseError::GRIB2WrongIteration(7))
        );
    }

    #[test]
    fn get_tmpl_code_normal() {
        let sect = SectionInfo {
            num: 5,
            offset: 8902,
            size: 23,
            body: Some(SectionBody::Section5(ReprDefinition {
                num_points: 86016,
                repr_tmpl_num: 200,
            })),
        };

        assert_eq!(sect.get_tmpl_code(), Some(TemplateInfo(5, 200)));
    }

    #[test]
    fn get_templates_normal() {
        let sects = vec![
            sect_placeholder!(1),
            SectionInfo {
                num: 3,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section3(GridDefinition {
                    num_points: 0,
                    grid_tmpl_num: 0,
                })),
            },
            SectionInfo {
                num: 4,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section4(ProdDefinition {
                    num_coordinates: 0,
                    prod_tmpl_num: 0,
                })),
            },
            SectionInfo {
                num: 5,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section5(ReprDefinition {
                    num_points: 0,
                    repr_tmpl_num: 0,
                })),
            },
            sect_placeholder!(6),
            sect_placeholder!(7),
            SectionInfo {
                num: 3,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section3(GridDefinition {
                    num_points: 0,
                    grid_tmpl_num: 1,
                })),
            },
            SectionInfo {
                num: 4,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section4(ProdDefinition {
                    num_coordinates: 0,
                    prod_tmpl_num: 0,
                })),
            },
            SectionInfo {
                num: 5,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section5(ReprDefinition {
                    num_points: 0,
                    repr_tmpl_num: 0,
                })),
            },
            sect_placeholder!(6),
            sect_placeholder!(7),
            sect_placeholder!(8),
        ]
        .into_boxed_slice();

        assert_eq!(
            get_templates(&sects),
            vec![
                TemplateInfo(3, 0),
                TemplateInfo(3, 1),
                TemplateInfo(4, 0),
                TemplateInfo(5, 0),
            ]
        );
    }
}
