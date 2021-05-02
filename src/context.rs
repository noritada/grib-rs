use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt::{self, Display, Formatter};
use std::io::{Read, Seek};
use std::result::Result;

use crate::codetables::{
    CodeTable3_1, CodeTable4_0, CodeTable4_1, CodeTable4_2, CodeTable4_3, CodeTable4_4,
    CodeTable5_0, Lookup,
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
    pub(crate) templated: Box<[u8]>,
    pub(crate) template_supported: bool,
}

impl ProdDefinition {
    pub fn parameter_category(&self) -> Option<&u8> {
        if self.template_supported {
            self.templated.get(0)
        } else {
            None
        }
    }

    pub fn parameter_number(&self) -> Option<&u8> {
        if self.template_supported {
            self.templated.get(1)
        } else {
            None
        }
    }

    pub fn generating_process(&self) -> Option<&u8> {
        if self.template_supported {
            let index = match self.prod_tmpl_num {
                0..=39 => Some(2),
                40..=43 => Some(4),
                44..=46 => Some(15),
                47 => Some(2),
                48..=49 => Some(26),
                51 => Some(2),
                // 53 and 54 is variable and not supported as of now
                55..=56 => Some(8),
                // 57 and 58 is variable and not supported as of now
                59 => Some(8),
                60..=61 => Some(2),
                62..=63 => Some(8),
                // 67 and 68 is variable and not supported as of now
                70..=73 => Some(7),
                76..=79 => Some(5),
                80..=81 => Some(27),
                82 => Some(16),
                83 => Some(2),
                84 => Some(16),
                85 => Some(15),
                86..=91 => Some(2),
                254 => Some(2),
                1000..=1101 => Some(2),
                _ => None,
            }?;
            self.templated.get(index)
        } else {
            None
        }
    }

    pub fn forecast_time(&self) -> Option<(u8, u32)> {
        if self.template_supported {
            let unit_index = match self.prod_tmpl_num {
                0..=15 => Some(8),
                32..=34 => Some(8),
                40..=43 => Some(10),
                44..=47 => Some(21),
                48..=49 => Some(32),
                51 => Some(8),
                // 53 and 54 is variable and not supported as of now
                55..=56 => Some(14),
                // 57 and 58 is variable and not supported as of now
                59 => Some(14),
                60..=61 => Some(8),
                62..=63 => Some(14),
                // 67 and 68 is variable and not supported as of now
                70..=73 => Some(13),
                76..=79 => Some(11),
                80..=81 => Some(33),
                82..=84 => Some(22),
                85 => Some(21),
                86..=87 => Some(8),
                88 => Some(26),
                91 => Some(8),
                1000..=1101 => Some(8),
                _ => None,
            }?;
            let unit = self.templated.get(unit_index).map(|v| *v);
            let start = unit_index + 1;
            let end = unit_index + 5;
            let time = u32::from_be_bytes(self.templated[start..end].try_into().unwrap());
            unit.zip(Some(time))
        } else {
            None
        }
    }
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
struct SubMessageIndex {
    section2: Option<usize>,
    section3: usize,
    section4: usize,
    section5: usize,
    section6: usize,
    section7: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TemplateInfo(pub u8, pub u16);

impl TemplateInfo {
    pub fn describe(&self) -> Option<String> {
        match self.0 {
            3 => Some(CodeTable3_1.lookup(usize::from(self.1)).to_string()),
            4 => Some(CodeTable4_0.lookup(usize::from(self.1)).to_string()),
            5 => Some(CodeTable5_0.lookup(usize::from(self.1)).to_string()),
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
    submessages: Box<[SubMessageIndex]>,
}

impl<R: Grib2Read> Grib2<R> {
    pub fn read(mut r: R) -> Result<Self, GribError> {
        let sects = r.scan()?;
        let submessages = index_submessages(&sects)?;
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

    pub fn submessages(&self) -> SubMessageIterator {
        SubMessageIterator::new(&self.submessages, &self.sections)
    }

    /// Decodes grid values of a surface specified by the index `i`.
    pub fn get_values(&self, i: usize) -> Result<Box<[f32]>, GribError> {
        let (sect5, sect6, sect7) = self
            .submessages
            .get(i)
            .and_then(|submsg| {
                Some((
                    self.sections.get(submsg.section5)?,
                    self.sections.get(submsg.section6)?,
                    self.sections.get(submsg.section7)?,
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

    pub fn list_templates(&self) -> Vec<TemplateInfo> {
        get_templates(&self.sections)
    }
}

/// Validates the section order of sections and split them into a
/// vector of section groups.
fn index_submessages(
    sects: &Box<[SectionInfo]>,
) -> Result<Box<[SubMessageIndex]>, ValidationError> {
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
            i3_default = Some(submessage.section3);
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
                update_default!(SubMessageIndex {
                    section2: Some(i),
                    section3: i3,
                    section4: i4,
                    section5: i5,
                    section6: i6,
                    section7: i7,
                })
            }
            Some((_i, SectionInfo { num: 3, .. })) => {
                let (i, _) = sect.unwrap();
                let i4 = check!(4);
                let i5 = check!(5);
                let i6 = check!(6);
                let i7 = check!(7);
                update_default!(SubMessageIndex {
                    section2: i2_default,
                    section3: i,
                    section4: i4,
                    section5: i5,
                    section6: i6,
                    section7: i7,
                })
            }
            Some((i, SectionInfo { num: 4, .. })) => {
                let i3 = i3_default.ok_or(ValidationError::NoGridDefinition(i))?;
                let (i, _) = sect.unwrap();
                let i5 = check!(5);
                let i6 = check!(6);
                let i7 = check!(7);
                update_default!(SubMessageIndex {
                    section2: i2_default,
                    section3: i3,
                    section4: i,
                    section5: i5,
                    section6: i6,
                    section7: i7,
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

#[derive(Clone)]
pub struct SubMessageIterator<'a> {
    indices: &'a Box<[SubMessageIndex]>,
    sections: &'a Box<[SectionInfo]>,
    pos: usize,
}

impl<'a> SubMessageIterator<'a> {
    fn new(indices: &'a Box<[SubMessageIndex]>, sections: &'a Box<[SectionInfo]>) -> Self {
        Self {
            indices: indices,
            sections: sections,
            pos: 0,
        }
    }

    fn new_submessage_section(&self, index: usize) -> Option<SubMessageSection<'a>> {
        Some(SubMessageSection::new(index, self.sections.get(index)?))
    }
}

impl<'a> Iterator for SubMessageIterator<'a> {
    type Item = SubMessage<'a>;

    fn next(&mut self) -> Option<SubMessage<'a>> {
        let submessage_index = self.indices.get(self.pos)?;
        self.pos += 1;

        Some(SubMessage(
            self.new_submessage_section(0)?,
            self.new_submessage_section(1)?,
            submessage_index
                .section2
                .and_then(|i| self.new_submessage_section(i)),
            self.new_submessage_section(submessage_index.section3)?,
            self.new_submessage_section(submessage_index.section4)?,
            self.new_submessage_section(submessage_index.section5)?,
            self.new_submessage_section(submessage_index.section6)?,
            self.new_submessage_section(submessage_index.section7)?,
            self.new_submessage_section(self.sections.len() - 1)?,
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.indices.len();
        (size, Some(size))
    }

    fn nth(&mut self, n: usize) -> Option<SubMessage<'a>> {
        self.pos = n;
        self.next()
    }
}

pub struct SubMessage<'a>(
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub Option<SubMessageSection<'a>>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
);

impl<'a> SubMessage<'a> {
    pub fn indicator(&self) -> &Indicator {
        // panics should not happen if data is correct
        match self.0.body.body.as_ref().unwrap() {
            SectionBody::Section0(data) => data,
            _ => panic!("something unexpected happened"),
        }
    }

    pub fn prod_def(&self) -> &ProdDefinition {
        // panics should not happen if data is correct
        match self.4.body.body.as_ref().unwrap() {
            SectionBody::Section4(data) => data,
            _ => panic!("something unexpected happened"),
        }
    }

    pub fn describe(&self) -> String {
        let category = self.prod_def().parameter_category();
        let forecast_time = self.prod_def().forecast_time();
        format!(
            "\
Grid:                                   {}
Product:                                {}
  Parameter Category:                   {}
  Parameter:                            {}
  Generating Proceess:                  {}
  Forecast Time:                        {}
  Forecast Time Unit:                   {}
Data Representation:                    {}
",
            self.3.describe().unwrap_or(String::new()),
            self.4.describe().unwrap_or(String::new()),
            category
                .map(|v| CodeTable4_1::new(self.indicator().discipline)
                    .lookup(usize::from(*v))
                    .to_string())
                .unwrap_or(String::new()),
            self.prod_def()
                .parameter_number()
                .zip(category)
                .map(|(n, c)| CodeTable4_2::new(self.indicator().discipline, *c)
                    .lookup(usize::from(*n))
                    .to_string())
                .unwrap_or(String::new()),
            self.prod_def()
                .generating_process()
                .map(|v| CodeTable4_3.lookup(usize::from(*v)).to_string())
                .unwrap_or(String::new()),
            forecast_time
                .map(|(_, v)| v.to_string())
                .unwrap_or(String::new()),
            forecast_time
                .map(|(unit, _)| CodeTable4_4.lookup(usize::from(unit)).to_string())
                .unwrap_or(String::new()),
            self.5.describe().unwrap_or(String::new()),
        )
    }
}

pub struct SubMessageSection<'a> {
    pub index: usize,
    pub body: &'a SectionInfo,
}

impl<'a> SubMessageSection<'a> {
    pub fn new(index: usize, body: &'a SectionInfo) -> Self {
        Self {
            index: index,
            body: body,
        }
    }

    pub fn template_code(&self) -> Option<TemplateInfo> {
        self.body.get_tmpl_code()
    }

    pub fn describe(&self) -> Option<String> {
        self.template_code().and_then(|code| code.describe())
    }
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
    fn index_submessages_simple() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Ok(vec![SubMessageIndex {
                section2: Some(2),
                section3: 3,
                section4: 4,
                section5: 5,
                section6: 6,
                section7: 7,
            },]
            .into_boxed_slice())
        );
    }

    #[test]
    fn index_submessages_sect2_loop() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Ok(vec![
                SubMessageIndex {
                    section2: Some(2),
                    section3: 3,
                    section4: 4,
                    section5: 5,
                    section6: 6,
                    section7: 7,
                },
                SubMessageIndex {
                    section2: Some(8),
                    section3: 9,
                    section4: 10,
                    section5: 11,
                    section6: 12,
                    section7: 13,
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn index_submessages_sect3_loop() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Ok(vec![
                SubMessageIndex {
                    section2: Some(2),
                    section3: 3,
                    section4: 4,
                    section5: 5,
                    section6: 6,
                    section7: 7,
                },
                SubMessageIndex {
                    section2: Some(2),
                    section3: 8,
                    section4: 9,
                    section5: 10,
                    section6: 11,
                    section7: 12,
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn index_submessages_sect3_loop_no_sect2() {
        let sects = sect_list![0, 1, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Ok(vec![
                SubMessageIndex {
                    section2: None,
                    section3: 2,
                    section4: 3,
                    section5: 4,
                    section6: 5,
                    section7: 6,
                },
                SubMessageIndex {
                    section2: None,
                    section3: 7,
                    section4: 8,
                    section5: 9,
                    section6: 10,
                    section7: 11,
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn index_submessages_sect4_loop() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Ok(vec![
                SubMessageIndex {
                    section2: Some(2),
                    section3: 3,
                    section4: 4,
                    section5: 5,
                    section6: 6,
                    section7: 7,
                },
                SubMessageIndex {
                    section2: Some(2),
                    section3: 3,
                    section4: 8,
                    section5: 9,
                    section6: 10,
                    section7: 11,
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn index_submessages_sect4_loop_no_sect2() {
        let sects = sect_list![0, 1, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Ok(vec![
                SubMessageIndex {
                    section2: None,
                    section3: 2,
                    section4: 3,
                    section5: 4,
                    section6: 5,
                    section7: 6,
                },
                SubMessageIndex {
                    section2: None,
                    section3: 2,
                    section4: 7,
                    section5: 8,
                    section6: 9,
                    section7: 10,
                },
            ]
            .into_boxed_slice())
        );
    }

    #[test]
    fn index_submessages_end_after_sect1() {
        let sects = sect_list![0, 1,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn index_submessages_end_in_sect2_loop_1() {
        let sects = sect_list![0, 1, 2,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn index_submessages_end_in_sect2_loop_2() {
        let sects = sect_list![0, 1, 2, 3,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn index_submessages_end_in_sect3_loop_1() {
        let sects = sect_list![0, 1, 3,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn index_submessages_end_in_sect3_loop_2() {
        let sects = sect_list![0, 1, 3, 4,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn index_submessages_end_in_sect4_loop_1() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 4,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn index_submessages_end_in_sect4_loop_2() {
        let sects = sect_list![0, 1, 2, 3, 4, 5, 6, 7, 4, 5,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2IterationSuddenlyFinished)
        );
    }

    #[test]
    fn index_submessages_no_grid_in_sect4() {
        let sects = sect_list![0, 1, 4, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::NoGridDefinition(2))
        );
    }

    #[test]
    fn index_submessages_no_grid_in_sect8() {
        let sects = sect_list![0, 1, 8,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::NoGridDefinition(2))
        );
    }

    #[test]
    fn index_submessages_wrong_order_in_sect2() {
        let sects = sect_list![0, 1, 2, 4, 3, 5, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2WrongIteration(3))
        );
    }

    #[test]
    fn index_submessages_wrong_order_in_sect3() {
        let sects = sect_list![0, 1, 3, 5, 4, 6, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
            Err(ValidationError::GRIB2WrongIteration(3))
        );
    }

    #[test]
    fn index_submessages_wrong_order_in_sect4() {
        let sects = sect_list![0, 1, 3, 4, 5, 6, 7, 4, 6, 5, 7, 8,];

        assert_eq!(
            index_submessages(&sects),
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
                    templated: Vec::new().into_boxed_slice(),
                    template_supported: true,
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
                    templated: Vec::new().into_boxed_slice(),
                    template_supported: true,
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

    #[test]
    fn prod_definition_parameters() {
        let data = ProdDefinition {
            num_coordinates: 0,
            prod_tmpl_num: 0,
            templated: vec![
                193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 40, 1, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255,
            ]
            .into_boxed_slice(),
            template_supported: true,
        };

        assert_eq!(data.parameter_category(), Some(&193));
        assert_eq!(data.parameter_number(), Some(&0));
        assert_eq!(data.forecast_time(), Some((0, 40)));
    }
}
