use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::io::{Read, Seek};
use std::result::Result;

use crate::codetables::{
    lookup_table, CODE_TABLE_0_0, CODE_TABLE_1_1, CODE_TABLE_1_2, CODE_TABLE_1_3, CODE_TABLE_1_4,
    CODE_TABLE_3_1, CODE_TABLE_4_0, CODE_TABLE_5_0, COMMON_CODE_TABLE_00, COMMON_CODE_TABLE_11,
};
use crate::decoder::{self, DecodeError};
use crate::reader::{Grib2Read, ParseError, SeekableGrib2Reader};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionInfo {
    pub num: u8,
    pub offset: usize,
    pub size: usize,
    pub body: Option<SectionBody>,
}

impl SectionInfo {
    pub fn get_tmpl_code(&self) -> Option<TemplateInfo> {
        let tmpl_num = self.body.as_ref()?.get_tmpl_num()?;
        Some(TemplateInfo(self.num, tmpl_num))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionBody {
    Section0(Indicator),
    Section1(Identification),
    Section2,
    Section3(GridDefinition),
    Section4(ProdDefinition),
    Section5(ReprDefinition),
    Section6(BitMap),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indicator {
    /// Discipline - GRIB Master Table Number (see Code Table 0.0)
    pub discipline: u8,
    /// Total length of GRIB message in octets (including Section 0)
    pub total_length: u64,
}

impl Display for Indicator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "\
Discipline:                             {}
Total Length:                           {}\
",
            lookup_table(CODE_TABLE_0_0, self.discipline as usize),
            self.total_length,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identification {
    /// Identification of originating/generating centre (see Common Code Table C-1)
    pub centre_id: u16,
    /// Identification of originating/generating sub-centre (allocated by originating/ generating centre)
    pub subcentre_id: u16,
    /// GRIB Master Tables Version Number (see Code Table 1.0)
    pub master_table_version: u8,
    /// GRIB Local Tables Version Number (see Code Table 1.1)
    pub local_table_version: u8,
    /// Significance of Reference Time (see Code Table 1.2)
    pub ref_time_significance: u8,
    /// Reference time of data
    pub ref_time: DateTime<Utc>,
    /// Production status of processed data in this GRIB message
    /// (see Code Table 1.3)
    pub prod_status: u8,
    /// Type of processed data in this GRIB message (see Code Table 1.4)
    pub data_type: u8,
}

impl Display for Identification {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "\
Originating/generating centre:          {}
Originating/generating sub-centre:      {}
GRIB Master Tables Version Number:      {} ({})
GRIB Local Tables Version Number:       {}
Significance of Reference Time:         {}
Reference time of data:                 {}
Production status of processed data:    {}
Type of processed data:                 {}\
",
            lookup_table(COMMON_CODE_TABLE_11, self.centre_id as usize),
            self.subcentre_id,
            self.master_table_version,
            lookup_table(COMMON_CODE_TABLE_00, self.master_table_version as usize),
            lookup_table(CODE_TABLE_1_1, self.local_table_version as usize),
            lookup_table(CODE_TABLE_1_2, self.ref_time_significance as usize),
            self.ref_time,
            lookup_table(CODE_TABLE_1_3, self.prod_status as usize),
            lookup_table(CODE_TABLE_1_4, self.data_type as usize)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GridDefinition {
    /// Number of data points
    pub num_points: u32,
    /// Grid Definition Template Number
    pub grid_tmpl_num: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProdDefinition {
    /// Number of coordinate values after Template
    pub num_coordinates: u16,
    /// Product Definition Template Number
    pub prod_tmpl_num: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReprDefinition {
    /// Number of data points where one or more values are
    /// specified in Section 7 when a bit map is present, total
    /// number of data points when a bit map is absent
    pub num_points: u32,
    /// Data Representation Template Number
    pub repr_tmpl_num: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitMap {
    /// Bit-map indicator
    pub bitmap_indicator: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubMessage {
    pub section2: Option<usize>,
    pub section3: Option<usize>,
    pub section4: Option<usize>,
    pub section5: Option<usize>,
    pub section6: Option<usize>,
    pub section7: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TemplateInfo(pub u8, pub u16);

impl TemplateInfo {
    pub fn describe(&self) -> Option<String> {
        match self.0 {
            3 => Some(lookup_table(CODE_TABLE_3_1, usize::from(self.1)).to_string()),
            4 => Some(lookup_table(CODE_TABLE_4_0, usize::from(self.1)).to_string()),
            5 => Some(lookup_table(CODE_TABLE_5_0, usize::from(self.1)).to_string()),
            _ => None,
        }
    }
}

impl Display for TemplateInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

pub struct Grib2<R> {
    reader: RefCell<R>,
    sections: Box<[SectionInfo]>,
    submessages: Box<[SubMessage]>,
}

impl<R: Grib2Read> Grib2<R> {
    pub fn read(mut r: R) -> Result<Self, GribError> {
        let sects = r.scan()?;
        let submessages = get_submessages(&sects)?;
        Ok(Self {
            reader: RefCell::new(r),
            sections: sects,
            submessages: submessages,
        })
    }

    pub fn read_with_seekable<SR: Read + Seek>(
        r: SR,
    ) -> Result<Grib2<SeekableGrib2Reader<SR>>, GribError> {
        let r = SeekableGrib2Reader::new(r);
        Grib2::<SeekableGrib2Reader<SR>>::read(r)
    }

    pub fn info(&self) -> Result<(&Indicator, &Identification), GribError> {
        match (self.sections.get(0), self.sections.get(1)) {
            (
                Some(SectionInfo {
                    body: Some(SectionBody::Section0(sect0_body)),
                    ..
                }),
                Some(SectionInfo {
                    body: Some(SectionBody::Section1(sect1_body)),
                    ..
                }),
            ) => Ok((sect0_body, sect1_body)),
            _ => Err(GribError::InternalDataError),
        }
    }

    pub fn submessages(&self) -> &Box<[SubMessage]> {
        &self.submessages
    }

    pub fn describe_submessage(&self, submessage: &SubMessage) -> String {
        let sect3 = submessage.section3;
        let sect4 = submessage.section4;
        let sect5 = submessage.section5;
        format!(
            "\
Grid:                                   {}
Product:                                {}
Data Representation:                    {}
",
            sect3
                .and_then(|id| self.describe_section(id))
                .unwrap_or(String::new()),
            sect4
                .and_then(|id| self.describe_section(id))
                .unwrap_or(String::new()),
            sect5
                .and_then(|id| self.describe_section(id))
                .unwrap_or(String::new()),
        )
    }

    /// Decodes grid values of a surface specified by the index `i`.
    pub fn get_values(&self, i: usize) -> Result<Box<[f32]>, GribError> {
        let (sect5, sect6, sect7) = self
            .submessages
            .get(i)
            .and_then(|submsg| {
                Some((
                    submsg.section5.and_then(|i| self.sections.get(i))?,
                    submsg.section6.and_then(|i| self.sections.get(i))?,
                    submsg.section7.and_then(|i| self.sections.get(i))?,
                ))
            })
            .ok_or(GribError::InternalDataError)?;

        let reader = self.reader.borrow_mut();
        let values = decoder::dispatch(sect5, sect6, sect7, reader)?;
        Ok(values)
    }

    pub fn sections(&self) -> &Box<[SectionInfo]> {
        &self.sections
    }

    pub fn describe_section(&self, section_id: usize) -> Option<String> {
        self.sections
            .get(section_id)
            .and_then(|s| s.get_tmpl_code()?.describe())
    }

    pub fn list_templates(&self) -> Vec<TemplateInfo> {
        get_templates(&self.sections)
    }
}

/// Validates the section order of sections and split them into a
/// vector of section groups.
fn get_submessages(sects: &Box<[SectionInfo]>) -> Result<Box<[SubMessage]>, ValidationError> {
    let mut iter = sects.iter().enumerate();
    let mut starts = Vec::new();
    let mut i2_default = None;
    let mut i3_default = None;

    macro_rules! check {
        ($num:expr) => {{
            let (i, sect) = iter
                .next()
                .ok_or(ValidationError::GRIB2IterationSuddenlyFinished)?;
            if sect.num != $num {
                return Err(ValidationError::GRIB2WrongIteration(i));
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

    check!(0);
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
                    return Err(ValidationError::NoGridDefinition(i));
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
                    return Err(ValidationError::NoGridDefinition(i));
                }
                if i < sects.len() - 1 {
                    return Err(ValidationError::GRIB2WrongIteration(i));
                }
                break;
            }
            Some((i, SectionInfo { .. })) => {
                return Err(ValidationError::GRIB2WrongIteration(i));
            }
            None => {
                return Err(ValidationError::GRIB2IterationSuddenlyFinished);
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GribError {
    InternalDataError,
    ParseError(ParseError),
    ValidationError(ValidationError),
    DecodeError(DecodeError),
}

impl From<ParseError> for GribError {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
}

impl From<ValidationError> for GribError {
    fn from(e: ValidationError) -> Self {
        Self::ValidationError(e)
    }
}

impl From<DecodeError> for GribError {
    fn from(e: DecodeError) -> Self {
        Self::DecodeError(e)
    }
}

impl Display for GribError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InternalDataError => write!(f, "Something unexpected happend"),
            Self::ParseError(e) => write!(f, "{}", e),
            Self::ValidationError(e) => write!(f, "{}", e),
            Self::DecodeError(e) => write!(f, "{:#?}", e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValidationError {
    GRIB2IterationSuddenlyFinished,
    NoGridDefinition(usize),
    GRIB2WrongIteration(usize),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::GRIB2IterationSuddenlyFinished => write!(f, "GRIB2 file suddenly finished"),
            Self::NoGridDefinition(i) => write!(f, "Grid Definition Section not found at {}", i),
            Self::GRIB2WrongIteration(i) => write!(f, "GRIB2 sections wrongly ordered at {}", i),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn get_submessages_simple() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![SubMessage {
                section2: Some(2),
                section3: Some(3),
                section4: Some(4),
                section5: Some(5),
                section6: Some(6),
                section7: Some(7),
            },]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_sect2_loop() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: Some(2),
                    section3: Some(3),
                    section4: Some(4),
                    section5: Some(5),
                    section6: Some(6),
                    section7: Some(7),
                },
                SubMessage {
                    section2: Some(8),
                    section3: Some(9),
                    section4: Some(10),
                    section5: Some(11),
                    section6: Some(12),
                    section7: Some(13),
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn get_submessages_sect3_loop() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: Some(2),
                    section3: Some(3),
                    section4: Some(4),
                    section5: Some(5),
                    section6: Some(6),
                    section7: Some(7),
                },
                SubMessage {
                    section2: Some(2),
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
    fn get_submessages_sect3_loop_no_sect2() {
        let sects = sect_list![0, 1, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: None,
                    section3: Some(2),
                    section4: Some(3),
                    section5: Some(4),
                    section6: Some(5),
                    section7: Some(6),
                },
                SubMessage {
                    section2: None,
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
    fn get_submessages_sect4_loop() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: Some(2),
                    section3: Some(3),
                    section4: Some(4),
                    section5: Some(5),
                    section6: Some(6),
                    section7: Some(7),
                },
                SubMessage {
                    section2: Some(2),
                    section3: Some(3),
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
    fn get_submessages_sect4_loop_no_sect2() {
        let sects = sect_list![0, 1, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Ok(vec![
                SubMessage {
                    section2: None,
                    section3: Some(2),
                    section4: Some(3),
                    section5: Some(4),
                    section6: Some(5),
                    section7: Some(6),
                },
                SubMessage {
                    section2: None,
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
    fn get_submessages_end_after_sect1() {
        let sects = sect_list![0, 1,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect2_loop_1() {
        let sects = sect_list![0, 1, 2,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect2_loop_2() {
        let sects = sect_list![0, 1, 2, 3,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect3_loop_1() {
        let sects = sect_list![0, 1, 3,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect3_loop_2() {
        let sects = sect_list![0, 1, 3, 4,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect4_loop_1() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 4,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_end_in_sect4_loop_2() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 4, 5,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn get_submessages_no_grid_in_sect4() {
        let sects = sect_list![0, 1, 4, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::NoGridDefinition(2))
        );
    }

    #[test]
    fn get_submessages_no_grid_in_sect8() {
        let sects = sect_list![0, 1, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::NoGridDefinition(2))
        );
    }

    #[test]
    fn get_submessages_wrong_order_in_sect2() {
        let sects = sect_list![0, 1, 2, 4, 3, 5, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2WrongIteration(3))
        );
    }

    #[test]
    fn get_submessages_wrong_order_in_sect3() {
        let sects = sect_list![0, 1, 3, 5, 4, 6, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2WrongIteration(3))
        );
    }

    #[test]
    fn get_submessages_wrong_order_in_sect4() {
        let sects = sect_list![0, 1, 3, 4, 5, 6, 7, 4, 6, 5, 7, 8,];

        assert_eq!(
            get_submessages(&sects),
            Err(ValidationError::GRIB2WrongIteration(8))
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
            sect_placeholder!(0),
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
