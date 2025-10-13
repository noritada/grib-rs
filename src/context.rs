use std::{
    cell::{RefCell, RefMut},
    collections::HashSet,
    fmt::{self, Display, Formatter},
    io::{Cursor, Read, Seek},
};

use grib_template_helpers::{Dump as _, TryFromSlice as _};

#[cfg(feature = "time-calculation")]
use crate::TemporalInfo;
use crate::{
    GridPointIndexIterator, TemporalRawInfo,
    codetables::{
        CodeTable3_1, CodeTable4_0, CodeTable4_1, CodeTable4_2, CodeTable4_3, CodeTable5_0, Lookup,
    },
    datatypes::*,
    def::grib2::{Section1, Section5, SectionHeader},
    error::*,
    grid::GridPointIterator,
    parser::Grib2SubmessageIndexStream,
    reader::{Grib2Read, Grib2SectionStream, SECT8_ES_SIZE, SeekableGrib2Reader},
};

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

/// Reads a [`Grib2`] instance from an I/O stream of GRIB2.
///
/// # Examples
///
/// ```
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let f = std::fs::File::open(
///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
///     )?;
///     let f = std::io::BufReader::new(f);
///     let result = grib::from_reader(f);
///
///     assert!(result.is_ok());
///     let grib2 = result?;
///     assert_eq!(grib2.len(), 1);
///     Ok(())
/// }
/// ```
pub fn from_reader<SR: Read + Seek>(
    reader: SR,
) -> Result<Grib2<SeekableGrib2Reader<SR>>, GribError> {
    Grib2::<SeekableGrib2Reader<SR>>::read_with_seekable(reader)
}

/// Reads a [`Grib2`] instance from bytes of GRIB2.
///
/// # Examples
///
/// You can use this method to create a reader from a slice, i.e., a borrowed
/// sequence of bytes:
///
/// ```
/// use std::io::Read;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let f = std::fs::File::open(
///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
///     )?;
///     let mut f = std::io::BufReader::new(f);
///     let mut buf = Vec::new();
///     f.read_to_end(&mut buf).unwrap();
///     let result = grib::from_bytes(&buf);
///
///     assert!(result.is_ok());
///     let grib2 = result?;
///     assert_eq!(grib2.len(), 1);
///     Ok(())
/// }
/// ```
///
/// Also, you can use this method to create a reader from an owned sequence of
/// bytes:
///
/// ```
/// use std::io::Read;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let f = std::fs::File::open(
///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
///     )?;
///     let mut f = std::io::BufReader::new(f);
///     let mut buf = Vec::new();
///     f.read_to_end(&mut buf).unwrap();
///     let result = grib::from_bytes(buf);
///
///     assert!(result.is_ok());
///     let grib2 = result?;
///     assert_eq!(grib2.len(), 1);
///     Ok(())
/// }
/// ```
pub fn from_bytes<T>(bytes: T) -> Result<Grib2<SeekableGrib2Reader<Cursor<T>>>, GribError>
where
    T: AsRef<[u8]>,
{
    let reader = Cursor::new(bytes);
    Grib2::<SeekableGrib2Reader<Cursor<T>>>::read_with_seekable(reader)
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
    pub fn iter(&self) -> SubmessageIterator<'_, R> {
        self.into_iter()
    }

    /// Returns an iterator over submessages in the data.
    ///
    /// This is an alias to [`Grib2::iter()`].
    pub fn submessages(&self) -> SubmessageIterator<'_, R> {
        self.into_iter()
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
    pub fn sections(&self) -> std::slice::Iter<'_, SectionInfo> {
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

impl<'a, R: 'a> IntoIterator for &'a Grib2<R> {
    type Item = (MessageIndex, SubMessage<'a, R>);
    type IntoIter = SubmessageIterator<'a, R>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

fn get_templates(sects: &[SectionInfo]) -> Vec<TemplateInfo> {
    let uniq: HashSet<_> = sects.iter().filter_map(|s| s.get_tmpl_code()).collect();
    let mut vec: Vec<_> = uniq.into_iter().collect();
    vec.sort_unstable();
    vec
}

/// An iterator over submessages in the GRIB data.
///
/// This `struct` is created by the [`iter`] method on [`Grib2`]. See its
/// documentation for more.
///
/// [`iter`]: Grib2::iter
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
                self.new_submessage_section(submessage_index.0)?,
                self.new_submessage_section(submessage_index.1)?,
                submessage_index
                    .2
                    .and_then(|i| self.new_submessage_section(i)),
                self.new_submessage_section(submessage_index.3)?,
                self.new_submessage_section(submessage_index.4)?,
                self.new_submessage_section(submessage_index.5)?,
                self.new_submessage_section(submessage_index.6)?,
                self.new_submessage_section(submessage_index.7)?,
                self.new_submessage_section(submessage_index.8)?,
                self.context.reader.borrow_mut(),
            ),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.context.submessages.len() - self.pos;
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

impl<R> SubMessage<'_, R> {
    /// Returns the product's parameter.
    ///
    /// In the context of GRIB products, parameters refer to weather elements
    /// such as air temperature, air pressure, and humidity, and other physical
    /// quantities.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::File,
    ///     io::{BufReader, Read},
    /// };
    ///
    /// use grib::codetables::NCEP;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut buf = Vec::new();
    ///
    ///     let f = File::open("testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz")?;
    ///     let f = BufReader::new(f);
    ///     let mut f = xz2::bufread::XzDecoder::new(f);
    ///     f.read_to_end(&mut buf)?;
    ///
    ///     let f = std::io::Cursor::new(buf);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.iter();
    ///     let (_, message) = iter.next().ok_or_else(|| "first message is not found")?;
    ///
    ///     let param = message.parameter();
    ///     assert_eq!(
    ///         param,
    ///         Some(grib::Parameter {
    ///             discipline: 0,
    ///             centre: 7,
    ///             master_ver: 2,
    ///             local_ver: 1,
    ///             category: 3,
    ///             num: 1
    ///         })
    ///     );
    ///     let param = param.unwrap();
    ///     assert_eq!(
    ///         param.description(),
    ///         Some("Pressure reduced to MSL".to_owned())
    ///     );
    ///     assert!(param.is_identical_to(NCEP::PRMSL));
    ///     Ok(())
    /// }
    /// ```
    pub fn parameter(&self) -> Option<Parameter> {
        let discipline = self.indicator().discipline;
        let ident = self.identification();
        let centre = ident.centre_id();
        let master_ver = ident.master_table_version();
        let local_ver = ident.local_table_version();
        let prod_def = self.prod_def();
        let category = prod_def.parameter_category()?;
        let num = prod_def.parameter_number()?;
        Some(Parameter {
            discipline,
            centre,
            master_ver,
            local_ver,
            category,
            num,
        })
    }

    pub fn indicator(&self) -> &Indicator {
        // panics should not happen if data is correct
        match self.0.body.body.as_ref().unwrap() {
            SectionBody::Section0(data) => data,
            _ => panic!("something unexpected happened"),
        }
    }

    pub fn identification(&self) -> &Identification {
        // panics should not happen if data is correct
        match self.1.body.body.as_ref().unwrap() {
            SectionBody::Section1(data) => data,
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

    /// Provides access to the parameters in Section 1.
    ///
    /// # Examples
    /// ```
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
    ///     )?;
    ///     let f = std::io::BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///     let (_index, first_submessage) = grib2.iter().next().unwrap();
    ///
    ///     let actual = first_submessage.section1();
    ///     let expected = Ok(grib::def::grib2::Section1 {
    ///         header: grib::def::grib2::SectionHeader {
    ///             len: 21,
    ///             sect_num: 1,
    ///         },
    ///         payload: grib::def::grib2::Section1Payload {
    ///             centre_id: 34,
    ///             subcentre_id: 0,
    ///             master_table_version: 5,
    ///             local_table_version: 1,
    ///             ref_time_significance: 0,
    ///             ref_time: grib::def::grib2::RefTime {
    ///                 year: 2016,
    ///                 month: 8,
    ///                 day: 22,
    ///                 hour: 2,
    ///                 minute: 0,
    ///                 second: 0,
    ///             },
    ///             prod_status: 0,
    ///             data_type: 2,
    ///             optional: None,
    ///         },
    ///     });
    ///     assert_eq!(actual, expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn section1(&self) -> Result<Section1, GribError> {
        let Identification { payload } = self.identification();
        let mut pos = 0;
        let payload = crate::def::grib2::Section1Payload::try_from_slice(payload, &mut pos)
            .map_err(|e| GribError::Unknown(e.to_owned()))?;

        let SectionInfo { num, size, .. } = self.1.body;
        Ok(Section1 {
            header: SectionHeader {
                len: *size as u32,
                sect_num: *num,
            },
            payload,
        })
    }

    /// Provides access to the parameters in Section 5.
    ///
    /// # Examples
    /// ```
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
    ///     )?;
    ///     let f = std::io::BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///     let (_index, first_submessage) = grib2.iter().next().unwrap();
    ///
    ///     let actual = first_submessage.section5();
    ///     let expected = Ok(grib::def::grib2::Section5 {
    ///         header: grib::def::grib2::SectionHeader {
    ///             len: 23,
    ///             sect_num: 5,
    ///         },
    ///         payload: grib::def::grib2::Section5Payload {
    ///             num_encoded_points: 86016,
    ///             template_num: 200,
    ///             template: grib::def::grib2::DataRepresentationTemplate::_5_200(
    ///                 grib::def::grib2::template::Template5_200 {
    ///                     num_bits: 8,
    ///                     max_val: 3,
    ///                     max_level: 3,
    ///                     dec: 0,
    ///                     level_vals: vec![1, 2, 3],
    ///                 },
    ///             ),
    ///         },
    ///     });
    ///     assert_eq!(actual, expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn section5(&self) -> Result<Section5, GribError> {
        let ReprDefinition { payload } = self.repr_def();
        let mut pos = 0;
        let payload = crate::def::grib2::Section5Payload::try_from_slice(payload, &mut pos)
            .map_err(|e| GribError::Unknown(e.to_owned()))?;

        let SectionInfo { num, size, .. } = self.5.body;
        Ok(Section5 {
            header: SectionHeader {
                len: *size as u32,
                sect_num: *num,
            },
            payload,
        })
    }

    /// Dumps the GRIB2 submessage.
    ///
    /// # Examples
    /// ```
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
    ///     )?;
    ///     let f = std::io::BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///     let (_index, first_submessage) = grib2.iter().next().unwrap();
    ///
    ///     let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
    ///     first_submessage.dump(&mut buf)?;
    ///     let expected = "\
    /// ##  SUBMESSAGE (total_length = 10321)
    /// ###  SECTION 0: INDICATOR SECTION (length = 16)
    /// ###  SECTION 1: IDENTIFICATION SECTION (length = 21)
    /// 1-4       header.len = 21  // Length of section in octets (nn).
    /// 5         header.sect_num = 1  // Number of section.
    /// 6-7       payload.centre_id = 34  // Identification of originating/generating centre (see Common Code table C–11).
    /// 8-9       payload.subcentre_id = 0  // Identification of originating/generating subcentre (allocated by originating/generating centre).
    /// 10        payload.master_table_version = 5  // GRIB master table version number (see Common Code table C–0 and Note 1).
    /// 11        payload.local_table_version = 1  // Version number of GRIB Local tables used to augment Master tables (see Code table 1.1 and Note 2).
    /// 12        payload.ref_time_significance = 0  // Significance of reference time (see Code table 1.2).
    /// 13-14     payload.ref_time.year = 2016  // Year (4 digits).
    /// 15        payload.ref_time.month = 8  // Month.
    /// 16        payload.ref_time.day = 22  // Day.
    /// 17        payload.ref_time.hour = 2  // Hour.
    /// 18        payload.ref_time.minute = 0  // Minute.
    /// 19        payload.ref_time.second = 0  // Second.
    /// 20        payload.prod_status = 0  // Production status of processed data in this GRIB message (see Code table 1.3).
    /// 21        payload.data_type = 2  // Type of processed data in this GRIB message (see Code table 1.4).
    /// ###  SECTION 3: GRID DEFINITION SECTION (length = 72)
    /// ###  SECTION 4: PRODUCT DEFINITION SECTION (length = 34)
    /// ###  SECTION 5: DATA REPRESENTATION SECTION (length = 23)
    /// 1-4       header.len = 23  // Length of section in octets (nn).
    /// 5         header.sect_num = 5  // Number of section.
    /// 6-9       payload.num_encoded_points = 86016  // Number of data points where one or more values are specified in Section 7 when a bit map is present, total number of data points when a bit map is absent.
    /// 10-11     payload.template_num = 200  // Data representation template number (see Code table 5.0).
    /// 12        payload.template.num_bits = 8  // Number of bits used for each packed value in the run length packing with level value.
    /// 13-14     payload.template.max_val = 3  // MV - maximum value within the levels that are used in the packing.
    /// 15-16     payload.template.max_level = 3  // MVL - maximum value of level (predefined).
    /// 17        payload.template.dec = 0  // Decimal scale factor of representative value of each level.
    /// 18-23     payload.template.level_vals = [1, 2, 3]  // List of MVL scaled representative values of each level from lv=1 to MVL.
    /// ###  SECTION 6: BIT-MAP SECTION (length = 6)
    /// ###  SECTION 7: DATA SECTION (length = 1391)
    /// ###  SECTION 8: END SECTION (length = 4)
    /// ";
    ///     assert_eq!(String::from_utf8_lossy(buf.get_ref()), expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn dump<W: std::io::Write>(&self, writer: &mut W) -> Result<(), GribError> {
        let write_heading =
            |writer: &mut W, sect: &SectionInfo, sect_name: &str| -> Result<(), std::io::Error> {
                let SectionInfo { num, size, .. } = sect;
                let sect_name = sect_name.to_ascii_uppercase();
                writeln!(writer, "##  SECTION {num}: {sect_name} (length = {size})")
            };

        let total_length = self.indicator().total_length;
        writeln!(writer, "#  SUBMESSAGE (total_length = {total_length})")?;
        write_heading(writer, self.0.body, "indicator section")?;
        write_heading(writer, self.1.body, "identification section")?;
        let mut pos = 1;
        self.section1()?.dump(None, &mut pos, writer)?;
        if let Some(sect) = &self.2 {
            write_heading(writer, sect.body, "local use section")?;
        }
        write_heading(writer, self.3.body, "grid definition section")?;
        write_heading(writer, self.4.body, "product definition section")?;
        write_heading(writer, self.5.body, "data representation section")?;
        let mut pos = 1;
        self.section5()?.dump(None, &mut pos, writer)?;
        write_heading(writer, self.6.body, "bit-map section")?;
        write_heading(writer, self.7.body, "data section")?;

        // Since `self.8.body` might be dummy, we don't use that Section 8 data.
        writeln!(writer, "##  SECTION 8: END SECTION (length = 4)")?;

        Ok(())
    }

    /// Returns time-related raw information associated with the submessage.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::File,
    ///     io::{BufReader, Read},
    /// };
    ///
    /// use grib::{
    ///     Code, ForecastTime, TemporalRawInfo, UtcDateTime,
    ///     codetables::grib2::{Table1_2, Table4_4},
    /// };
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = File::open(
    ///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
    ///     )?;
    ///     let f = BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.iter();
    ///
    ///     {
    ///         let (_, message) = iter.next().ok_or_else(|| "first message is not found")?;
    ///         let actual = message.temporal_raw_info();
    ///         let expected = TemporalRawInfo {
    ///             ref_time_significance: Code::Name(Table1_2::Analysis),
    ///             ref_time_unchecked: UtcDateTime::new(2016, 8, 22, 2, 0, 0),
    ///             forecast_time_diff: Some(ForecastTime {
    ///                 unit: Code::Name(Table4_4::Minute),
    ///                 value: 0,
    ///             }),
    ///         };
    ///         assert_eq!(actual, expected);
    ///     }
    ///
    ///     {
    ///         let (_, message) = iter.next().ok_or_else(|| "second message is not found")?;
    ///         let actual = message.temporal_raw_info();
    ///         let expected = TemporalRawInfo {
    ///             ref_time_significance: Code::Name(Table1_2::Analysis),
    ///             ref_time_unchecked: UtcDateTime::new(2016, 8, 22, 2, 0, 0),
    ///             forecast_time_diff: Some(ForecastTime {
    ///                 unit: Code::Name(Table4_4::Minute),
    ///                 value: 10,
    ///             }),
    ///         };
    ///         assert_eq!(actual, expected);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn temporal_raw_info(&self) -> TemporalRawInfo {
        let ref_time_significance = self.identification().ref_time_significance();
        let ref_time_unchecked = self.identification().ref_time_unchecked();
        let forecast_time = self.prod_def().forecast_time();
        TemporalRawInfo::new(ref_time_significance, ref_time_unchecked, forecast_time)
    }

    #[cfg(feature = "time-calculation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "time-calculation")))]
    /// Returns time-related calculated information associated with the
    /// submessage.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::File,
    ///     io::{BufReader, Read},
    /// };
    ///
    /// use chrono::{TimeZone, Utc};
    /// use grib::{
    ///     Code, ForecastTime, TemporalInfo, UtcDateTime,
    ///     codetables::grib2::{Table1_2, Table4_4},
    /// };
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = File::open(
    ///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
    ///     )?;
    ///     let f = BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.iter();
    ///
    ///     {
    ///         let (_, message) = iter.next().ok_or_else(|| "first message is not found")?;
    ///         let actual = message.temporal_info();
    ///         let expected = TemporalInfo {
    ///             ref_time: Some(Utc.with_ymd_and_hms(2016, 8, 22, 2, 0, 0).unwrap()),
    ///             forecast_time_target: Some(Utc.with_ymd_and_hms(2016, 8, 22, 2, 0, 0).unwrap()),
    ///         };
    ///         assert_eq!(actual, expected);
    ///     }
    ///
    ///     {
    ///         let (_, message) = iter.next().ok_or_else(|| "second message is not found")?;
    ///         let actual = message.temporal_info();
    ///         let expected = TemporalInfo {
    ///             ref_time: Some(Utc.with_ymd_and_hms(2016, 8, 22, 2, 0, 0).unwrap()),
    ///             forecast_time_target: Some(Utc.with_ymd_and_hms(2016, 8, 22, 2, 10, 0).unwrap()),
    ///         };
    ///         assert_eq!(actual, expected);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn temporal_info(&self) -> TemporalInfo {
        let raw_info = self.temporal_raw_info();
        TemporalInfo::from(&raw_info)
    }

    /// Returns the shape of the grid, i.e. a tuple of the number of grids in
    /// the i and j directions.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::File,
    ///     io::{BufReader, Read},
    /// };
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut buf = Vec::new();
    ///
    ///     let f = File::open("testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz")?;
    ///     let f = BufReader::new(f);
    ///     let mut f = xz2::bufread::XzDecoder::new(f);
    ///     f.read_to_end(&mut buf)?;
    ///
    ///     let f = std::io::Cursor::new(buf);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.iter();
    ///     let (_, message) = iter.next().ok_or_else(|| "first message is not found")?;
    ///
    ///     let shape = message.grid_shape()?;
    ///     assert_eq!(shape, (1440, 721));
    ///     Ok(())
    /// }
    /// ```
    pub fn grid_shape(&self) -> Result<(usize, usize), GribError> {
        let grid_def = self.grid_def();
        let shape = GridDefinitionTemplateValues::try_from(grid_def)?.grid_shape();
        Ok(shape)
    }

    /// Computes and returns an iterator over `(i, j)` of grid points.
    ///
    /// The order of items is the same as the order of the grid point values,
    /// defined by the scanning mode ([`ScanningMode`](`crate::ScanningMode`))
    /// in the data.
    ///
    /// This iterator allows users to perform their own coordinate calculations
    /// for unsupported grid systems and map the results to grid point values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::File,
    ///     io::{BufReader, Read},
    /// };
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut buf = Vec::new();
    ///
    ///     let f = File::open("testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz")?;
    ///     let f = BufReader::new(f);
    ///     let mut f = xz2::bufread::XzDecoder::new(f);
    ///     f.read_to_end(&mut buf)?;
    ///
    ///     let f = std::io::Cursor::new(buf);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.iter();
    ///     let (_, message) = iter.next().ok_or_else(|| "first message is not found")?;
    ///
    ///     let mut latlons = message.ij()?;
    ///     assert_eq!(latlons.next(), Some((0, 0)));
    ///     assert_eq!(latlons.next(), Some((1, 0)));
    ///     Ok(())
    /// }
    /// ```
    pub fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        let grid_def = self.grid_def();
        let num_defined = grid_def.num_points() as usize;
        let ij = GridDefinitionTemplateValues::try_from(grid_def)?.ij()?;
        let (num_decoded, _) = ij.size_hint();
        if num_defined == num_decoded {
            Ok(ij)
        } else {
            Err(GribError::InvalidValueError(format!(
                "number of grid points does not match: {num_defined} (defined) vs {num_decoded} (decoded)"
            )))
        }
    }

    /// Computes and returns an iterator over latitudes and longitudes of grid
    /// points.
    ///
    /// The order of lat/lon data of grid points is the same as the order of the
    /// grid point values, defined by the scanning mode
    /// ([`ScanningMode`](`crate::ScanningMode`)) in the data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{
    ///     fs::File,
    ///     io::{BufReader, Read},
    /// };
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut buf = Vec::new();
    ///
    ///     let f = File::open("testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz")?;
    ///     let f = BufReader::new(f);
    ///     let mut f = xz2::bufread::XzDecoder::new(f);
    ///     f.read_to_end(&mut buf)?;
    ///
    ///     let f = std::io::Cursor::new(buf);
    ///     let grib2 = grib::from_reader(f)?;
    ///
    ///     let mut iter = grib2.iter();
    ///     let (_, message) = iter.next().ok_or_else(|| "first message is not found")?;
    ///
    ///     let mut latlons = message.latlons()?;
    ///     assert_eq!(latlons.next(), Some((90.0, 0.0)));
    ///     assert_eq!(latlons.next(), Some((90.0, 0.25000003)));
    ///     Ok(())
    /// }
    /// ```
    pub fn latlons(&self) -> Result<GridPointIterator, GribError> {
        let grid_def = self.grid_def();
        let num_defined = grid_def.num_points() as usize;
        let latlons = GridDefinitionTemplateValues::try_from(grid_def)?.latlons()?;
        let (num_decoded, _) = latlons.size_hint();
        if num_defined == num_decoded {
            Ok(latlons)
        } else {
            Err(GribError::InvalidValueError(format!(
                "number of grid points does not match: {num_defined} (defined) vs {num_decoded} (decoded)"
            )))
        }
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
    use std::{fs::File, io::BufReader};

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

    #[test]
    fn context_from_buf_reader() {
        let f = File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )
        .unwrap();
        let f = BufReader::new(f);
        let result = from_reader(f);
        assert!(result.is_ok())
    }

    #[test]
    fn context_from_bytes() {
        let f = File::open(
            "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
        )
        .unwrap();
        let mut f = BufReader::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        let result = from_bytes(&buf);
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

    macro_rules! test_submessage_iterator {
        ($((
            $name:ident,
            $xz_compressed_input:expr,
            $nth:expr,
            $expected_index:expr,
            $expected_section_indices:expr,
        ),)*) => ($(
            #[test]
            fn $name() -> Result<(), Box<dyn std::error::Error>> {
                let mut buf = Vec::new();

                let f = File::open($xz_compressed_input)?;
                let f = BufReader::new(f);
                let mut f = xz2::bufread::XzDecoder::new(f);
                f.read_to_end(&mut buf)?;

                let f = Cursor::new(buf);
                let grib2 = crate::from_reader(f)?;
                let mut iter = grib2.iter();

                let (actual_index, message) = iter.nth($nth).ok_or_else(|| "item not available")?;
                assert_eq!(actual_index, $expected_index);
                let actual_section_indices = get_section_indices(message);
                assert_eq!(actual_section_indices, $expected_section_indices);

                Ok(())
            }
        )*);
    }

    test_submessage_iterator! {
        (
            item_0_from_submessage_iterator_for_single_message_data_with_multiple_submessages,
            "testdata/Z__C_RJTD_20190304000000_MSM_GUID_Rjp_P-all_FH03-39_Toorg_grib2.bin.xz",
            0,
            (0, 0),
            (0, 1, None, 2, 3, 4, 5, 6, 0),
        ),
        (
            item_1_from_submessage_iterator_for_single_message_data_with_multiple_submessages,
            "testdata/Z__C_RJTD_20190304000000_MSM_GUID_Rjp_P-all_FH03-39_Toorg_grib2.bin.xz",
            1,
            (0, 1),
            (0, 1, None, 2, 7, 8, 9, 10, 0),
        ),
        (
            item_0_from_submessage_iterator_for_multi_message_data,
            "testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz",
            0,
            (0, 0),
            (0, 1, None, 2, 3, 4, 5, 6, 7),
        ),
        (
            item_1_from_submessage_iterator_for_multi_message_data,
            "testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz",
            1,
            (1, 0),
            (8, 9, None, 10, 11, 12, 13, 14, 15),
        ),
    }

    fn get_section_indices<R>(
        submessage: SubMessage<'_, R>,
    ) -> (
        usize,
        usize,
        Option<usize>,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ) {
        (
            submessage.0.index,
            submessage.1.index,
            submessage.2.map(|s| s.index),
            submessage.3.index,
            submessage.4.index,
            submessage.5.index,
            submessage.6.index,
            submessage.7.index,
            submessage.8.index,
        )
    }
}
