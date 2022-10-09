use std::cell::{RefCell, RefMut};
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::io::{Cursor, Read, Seek};
use std::result::Result;

use crate::codetables::{
    CodeTable3_1, CodeTable4_0, CodeTable4_1, CodeTable4_2, CodeTable4_3, CodeTable5_0, Lookup,
};
use crate::datatypes::*;
use crate::error::*;
use crate::parser::Grib2SubmessageIndexStream;
use crate::reader::{Grib2Read, Grib2SectionStream, SeekableGrib2Reader, SECT8_ES_SIZE};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
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

    pub(crate) fn new_8(offset: usize) -> Self {
        Self {
            num: 8,
            offset,
            size: SECT8_ES_SIZE,
            body: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionBody {
    Section0(Indicator),
    Section1(Identification),
    Section2(LocalUse),
    Section3(GridDefinition),
    Section4(ProdDefinition),
    Section5(ReprDefinition),
    Section6(BitMap),
    Section7,
}

impl SectionBody {
    fn get_tmpl_num(&self) -> Option<u16> {
        match self {
            Self::Section3(s) => Some(s.grid_tmpl_num()),
            Self::Section4(s) => Some(s.prod_tmpl_num()),
            Self::Section5(s) => Some(s.repr_tmpl_num()),
            _ => None,
        }
    }
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
    reader: RefCell<R>,
    sections: Box<[SectionInfo]>,
    submessages: Vec<Grib2SubmessageIndex>,
}

impl<R> Grib2<R> {
    /// Returns the length of submessages in the data.
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
    ///     )?;
    ///     let f = std::io::BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     assert_eq!(grib2.len(), 1);
    ///     Ok(())
    /// }
    /// ```
    pub fn len(&self) -> usize {
        self.submessages.len()
    }

    /// Returns `true` if `self` has zero submessages.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over submessages in the data.
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
    ///     )?;
    ///     let f = std::io::BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.iter();
    ///     let first = iter.next();
    ///     assert!(first.is_some());
    ///
    ///     let first = first.unwrap();
    ///     let (message_index, _) = first;
    ///     assert_eq!(message_index, (0, 0));
    ///
    ///     let second = iter.next();
    ///     assert!(second.is_none());
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn iter(&self) -> SubmessageIterator<R> {
        self.submessages()
    }

    /// Returns an iterator over submessages in the data.
    ///
    /// This is an alias to [`Grib2::iter()`].
    pub fn submessages(&self) -> SubmessageIterator<R> {
        SubmessageIterator::new(self)
    }

    /// Returns an iterator over sections in the data.
    ///
    /// # Examples
    ///
    /// ```
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
    ///     )?;
    ///     let f = std::io::BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.sections();
    ///     let first = iter.next();
    ///     assert!(first.is_some());
    ///
    ///     let first = first.unwrap();
    ///     assert_eq!(first.num, 0);
    ///
    ///     let tenth = iter.nth(9);
    ///     assert!(tenth.is_none());
    ///     Ok(())
    /// }
    /// ```
    pub fn sections(&self) -> std::slice::Iter<SectionInfo> {
        self.sections.iter()
    }
}

impl<R: Grib2Read> Grib2<R> {
    pub fn read(r: R) -> Result<Self, GribError> {
        let mut sect_stream = Grib2SectionStream::new(r);
        let mut cacher = Vec::new();
        let parser = Grib2SubmessageIndexStream::new(sect_stream.by_ref()).with_cacher(&mut cacher);
        let submessages = parser.collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            reader: RefCell::new(sect_stream.into_reader()),
            sections: cacher.into_boxed_slice(),
            submessages,
        })
    }

    pub fn read_with_seekable<SR: Read + Seek>(
        r: SR,
    ) -> Result<Grib2<SeekableGrib2Reader<SR>>, GribError> {
        let r = SeekableGrib2Reader::new(r);
        Grib2::<SeekableGrib2Reader<SR>>::read(r)
    }

    pub fn list_templates(&self) -> Vec<TemplateInfo> {
        get_templates(&self.sections)
    }
}

fn get_templates(sects: &[SectionInfo]) -> Vec<TemplateInfo> {
    let uniq: HashSet<_> = sects.iter().filter_map(|s| s.get_tmpl_code()).collect();
    let mut vec: Vec<_> = uniq.into_iter().collect();
    vec.sort_unstable();
    vec
}

#[derive(Clone)]
pub struct SubmessageIterator<'a, R> {
    context: &'a Grib2<R>,
    pos: usize,
}

impl<'a, R> SubmessageIterator<'a, R> {
    fn new(context: &'a Grib2<R>) -> Self {
        Self { context, pos: 0 }
    }

    fn new_submessage_section(&self, index: usize) -> Option<SubMessageSection<'a>> {
        Some(SubMessageSection::new(
            index,
            self.context.sections.get(index)?,
        ))
    }
}

impl<'a, R> Iterator for SubmessageIterator<'a, R> {
    type Item = (MessageIndex, SubMessage<'a, R>);

    fn next(&mut self) -> Option<Self::Item> {
        let submessage_index = self.context.submessages.get(self.pos)?;
        self.pos += 1;

        Some((
            submessage_index.message_index(),
            SubMessage(
                self.new_submessage_section(0)?,
                self.new_submessage_section(1)?,
                submessage_index
                    .2
                    .and_then(|i| self.new_submessage_section(i)),
                self.new_submessage_section(submessage_index.3)?,
                self.new_submessage_section(submessage_index.4)?,
                self.new_submessage_section(submessage_index.5)?,
                self.new_submessage_section(submessage_index.6)?,
                self.new_submessage_section(submessage_index.7)?,
                self.new_submessage_section(self.context.sections.len() - 1)?,
                self.context.reader.borrow_mut(),
            ),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.context.submessages.len();
        (size, Some(size))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.pos = n;
        self.next()
    }
}

impl<'a, R> IntoIterator for &'a SubmessageIterator<'a, R> {
    type Item = (MessageIndex, SubMessage<'a, R>);
    type IntoIter = SubmessageIterator<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        SubmessageIterator {
            context: self.context,
            pos: self.pos,
        }
    }
}

pub struct SubMessage<'a, R>(
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub Option<SubMessageSection<'a>>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub SubMessageSection<'a>,
    pub(crate) RefMut<'a, R>,
);

impl<'a, R> SubMessage<'a, R> {
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
            self.grid_def().num_points(),
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
            self.repr_def().num_points(),
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
    fn get_tmpl_code_normal() {
        let sect = SectionInfo {
            num: 5,
            offset: 8902,
            size: 23,
            body: Some(SectionBody::Section5(
                ReprDefinition::from_payload(
                    vec![0x00, 0x01, 0x50, 0x00, 0x00, 0xc8].into_boxed_slice(),
                )
                .unwrap(),
            )),
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
                body: Some(SectionBody::Section3(
                    GridDefinition::from_payload(vec![0; 9].into_boxed_slice()).unwrap(),
                )),
            },
            SectionInfo {
                num: 4,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section4(
                    ProdDefinition::from_payload(vec![0; 4].into_boxed_slice()).unwrap(),
                )),
            },
            SectionInfo {
                num: 5,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section5(
                    ReprDefinition::from_payload(vec![0; 6].into_boxed_slice()).unwrap(),
                )),
            },
            sect_placeholder!(6),
            sect_placeholder!(7),
            SectionInfo {
                num: 3,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section3(
                    GridDefinition::from_payload(
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 1].into_boxed_slice(),
                    )
                    .unwrap(),
                )),
            },
            SectionInfo {
                num: 4,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section4(
                    ProdDefinition::from_payload(vec![0; 4].into_boxed_slice()).unwrap(),
                )),
            },
            SectionInfo {
                num: 5,
                offset: 0,
                size: 0,
                body: Some(SectionBody::Section5(
                    ReprDefinition::from_payload(vec![0; 6].into_boxed_slice()).unwrap(),
                )),
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
