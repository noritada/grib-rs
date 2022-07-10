use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::io::{Cursor, Read, Seek};
use std::result::Result;

use crate::codetables::{
    CodeTable3_1, CodeTable4_0, CodeTable4_1, CodeTable4_2, CodeTable4_3, CodeTable5_0, Lookup,
};
use crate::datatypes::*;
use crate::decoders;
use crate::error::*;
use crate::reader::{Grib2Read, SeekableGrib2Reader};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SubMessageIndex {
    pub(crate) section2: Option<usize>,
    pub(crate) section3: usize,
    pub(crate) section4: usize,
    pub(crate) section5: usize,
    pub(crate) section6: usize,
    pub(crate) section7: usize,
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

pub fn from_reader<SR: Read + Seek>(
    reader: SR,
) -> Result<Grib2<SeekableGrib2Reader<SR>>, GribError> {
    Grib2::<SeekableGrib2Reader<SR>>::read_with_seekable(reader)
}

pub fn from_slice(bytes: &[u8]) -> Result<Grib2<SeekableGrib2Reader<Cursor<&[u8]>>>, GribError> {
    let reader = Cursor::new(bytes);
    Grib2::<SeekableGrib2Reader<Cursor<&[u8]>>>::read_with_seekable(reader)
}

pub struct Grib2<R> {
    pub(crate) reader: RefCell<R>,
    pub(crate) sections: Box<[SectionInfo]>,
    pub(crate) submessages: Box<[SubMessageIndex]>,
}

impl<R: Grib2Read> Grib2<R> {
    pub fn read(mut r: R) -> Result<Self, GribError> {
        let sections = r.scan()?;
        let submessages = index_submessages(&sections)?;
        Ok(Self {
            reader: RefCell::new(r),
            sections,
            submessages,
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

    /// Iterates over submessages.
    #[inline]
    pub fn iter(&self) -> SubMessageIterator {
        self.submessages()
    }

    pub fn submessages(&self) -> SubMessageIterator {
        SubMessageIterator::new(&self.submessages, &self.sections)
    }

    /// Decodes grid values of a surface specified by the index `i`.
    pub fn get_values(&self, i: usize) -> Result<Box<[f32]>, GribError> {
        let (sect3, sect5, sect6, sect7) = self
            .submessages
            .get(i)
            .and_then(|submsg| {
                Some((
                    self.sections.get(submsg.section3)?,
                    self.sections.get(submsg.section5)?,
                    self.sections.get(submsg.section6)?,
                    self.sections.get(submsg.section7)?,
                ))
            })
            .ok_or(GribError::InternalDataError)?;

        let reader = self.reader.borrow_mut();
        let values = decoders::dispatch(sect3, sect5, sect6, sect7, reader)?;
        Ok(values)
    }

    pub fn sections(&self) -> &[SectionInfo] {
        &self.sections
    }

    pub fn list_templates(&self) -> Vec<TemplateInfo> {
        get_templates(&self.sections)
    }
}

/// Validates the section order of sections and split them into a
/// vector of section groups.
fn index_submessages(sects: &[SectionInfo]) -> Result<Box<[SubMessageIndex]>, ValidationError> {
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

fn get_templates(sects: &[SectionInfo]) -> Vec<TemplateInfo> {
    let uniq: HashSet<_> = sects.iter().filter_map(|s| s.get_tmpl_code()).collect();
    let mut vec: Vec<_> = uniq.into_iter().collect();
    vec.sort_unstable();
    vec
}

#[derive(Clone)]
pub struct SubMessageIterator<'a> {
    indices: &'a [SubMessageIndex],
    sections: &'a [SectionInfo],
    pos: usize,
}

impl<'a> SubMessageIterator<'a> {
    fn new(indices: &'a [SubMessageIndex], sections: &'a [SectionInfo]) -> Self {
        Self {
            indices,
            sections,
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

    pub fn grid_def(&self) -> &GridDefinition {
        // panics should not happen if data is correct
        match self.3.body.body.as_ref().unwrap() {
            SectionBody::Section3(data) => data,
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

    pub fn repr_def(&self) -> &ReprDefinition {
        // panics should not happen if data is correct
        match self.5.body.body.as_ref().unwrap() {
            SectionBody::Section5(data) => data,
            _ => panic!("something unexpected happened"),
        }
    }

    pub fn describe(&self) -> String {
        let category = self.prod_def().parameter_category();
        let forecast_time = self
            .prod_def()
            .forecast_time()
            .map(|ft| ft.describe())
            .unwrap_or((String::new(), String::new()));
        let fixed_surfaces_info = self
            .prod_def()
            .fixed_surfaces()
            .map(|(first, second)| (first.describe(), second.describe()))
            .map(|(first, second)| (first.0, first.1, first.2, second.0, second.1, second.2))
            .unwrap_or((
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            ));

        format!(
            "\
Grid:                                   {}
  Number of points:                     {}
Product:                                {}
  Parameter Category:                   {}
  Parameter:                            {}
  Generating Proceess:                  {}
  Forecast Time:                        {}
  Forecast Time Unit:                   {}
  1st Fixed Surface Type:               {}
  1st Scale Factor:                     {}
  1st Scaled Value:                     {}
  2nd Fixed Surface Type:               {}
  2nd Scale Factor:                     {}
  2nd Scaled Value:                     {}
Data Representation:                    {}
  Number of represented values:         {}
",
            self.3.describe().unwrap_or_default(),
            self.grid_def().num_points,
            self.4.describe().unwrap_or_default(),
            category
                .map(|v| CodeTable4_1::new(self.indicator().discipline)
                    .lookup(usize::from(v))
                    .to_string())
                .unwrap_or_default(),
            self.prod_def()
                .parameter_number()
                .zip(category)
                .map(|(n, c)| CodeTable4_2::new(self.indicator().discipline, c)
                    .lookup(usize::from(n))
                    .to_string())
                .unwrap_or_default(),
            self.prod_def()
                .generating_process()
                .map(|v| CodeTable4_3.lookup(usize::from(v)).to_string())
                .unwrap_or_default(),
            forecast_time.1,
            forecast_time.0,
            fixed_surfaces_info.0,
            fixed_surfaces_info.1,
            fixed_surfaces_info.2,
            fixed_surfaces_info.3,
            fixed_surfaces_info.4,
            fixed_surfaces_info.5,
            self.5.describe().unwrap_or_default(),
            self.repr_def().num_points,
        )
    }
}

pub struct SubMessageSection<'a> {
    pub index: usize,
    pub body: &'a SectionInfo,
}

impl<'a> SubMessageSection<'a> {
    pub fn new(index: usize, body: &'a SectionInfo) -> Self {
        Self { index, body }
    }

    pub fn template_code(&self) -> Option<TemplateInfo> {
        self.body.get_tmpl_code()
    }

    pub fn describe(&self) -> Option<String> {
        self.template_code().and_then(|code| code.describe())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::BufReader;

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
    fn from_buf_reader() {
        let f = File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )
        .unwrap();
        let f = BufReader::new(f);
        let result = from_reader(f);
        assert!(result.is_ok())
    }

    #[test]
    fn from_bytes() {
        let f = File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )
        .unwrap();
        let mut f = BufReader::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        let result = from_slice(&buf);
        assert!(result.is_ok())
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
}
