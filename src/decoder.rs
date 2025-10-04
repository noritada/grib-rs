use std::vec::IntoIter;

use grib_template_helpers::TryFromSlice as _;

use crate::{
    context::{SectionBody, SubMessage},
    decoder::{
        bitmap::{BitmapDecodeIterator, dummy_bitmap_for_nonnullable_data},
        complex::ComplexPackingDecoded,
        simple::SimplePackingDecoder,
        stream::NBitwiseIterator,
    },
    def::grib2::{Section5Param, Template},
    error::*,
    reader::Grib2Read,
};

/// Decoder for grid point values of GRIB2 submessages.
///
/// # Examples
/// ```
/// use grib::Grib2SubmessageDecoder;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let f = std::fs::File::open(
///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
///     )?;
///     let f = std::io::BufReader::new(f);
///     let grib2 = grib::from_reader(f)?;
///     let (_index, first_submessage) = grib2.iter().next().unwrap();
///
///     let decoder = Grib2SubmessageDecoder::from(first_submessage)?;
///     let mut decoded = decoder.dispatch()?;
///     assert_eq!(decoded.size_hint(), (86016, Some(86016)));
///
///     let first_value = decoded.next();
///     assert_eq!(first_value.map(|f| f.is_nan()), Some(true));
///
///     let non_nan_value = decoded.find(|f| !f.is_nan());
///     assert_eq!(non_nan_value.map(|f| f.round()), Some(1.0_f32));
///
///     let last_value = decoded.last();
///     assert_eq!(last_value.map(|f| f.is_nan()), Some(true));
///     Ok(())
/// }
/// ```
///
/// If the byte sequences for Sections 5, 6, and 7 of the GRIB2 data are known,
/// and the number of grid points (described in Section 3) is also known, it is
/// also possible to create a decoder instance by passing them to
/// [`Grib2SubmessageDecoder::new`]. The example above is equivalent to the
/// following:
///
/// ```
/// use std::io::Read;
///
/// use grib::Grib2SubmessageDecoder;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let f = std::fs::File::open(
///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
///     )?;
///     let mut f = std::io::BufReader::new(f);
///     let mut buf = Vec::new();
///     f.read_to_end(&mut buf)?;
///
///     let decoder = Grib2SubmessageDecoder::new(
///         86016,
///         buf[0x0000008f..0x000000a6].to_vec(),
///         buf[0x000000a6..0x000000ac].to_vec(),
///         buf[0x000000ac..0x0000061b].to_vec(),
///     )?;
///     let mut decoded = decoder.dispatch()?;
///     assert_eq!(decoded.size_hint(), (86016, Some(86016)));
///
///     let first_value = decoded.next();
///     assert_eq!(first_value.map(|f| f.is_nan()), Some(true));
///
///     let non_nan_value = decoded.find(|f| !f.is_nan());
///     assert_eq!(non_nan_value.map(|f| f.round()), Some(1.0_f32));
///
///     let last_value = decoded.last();
///     assert_eq!(last_value.map(|f| f.is_nan()), Some(true));
///     Ok(())
/// }
/// ```
pub struct Grib2SubmessageDecoder {
    num_points_total: usize,
    sect5_param: Section5Param,
    sect6_bytes: Vec<u8>,
    sect7_bytes: Vec<u8>,
}

impl Grib2SubmessageDecoder {
    /// Creates an instance from the number of grid points (described in Section
    /// 3) and byte sequences for Sections 5, 6, and 7 of the GRIB2 data.
    ///
    /// For code examples, refer to the description of this `struct`.
    pub fn new(
        num_points_total: usize,
        sect5_bytes: Vec<u8>,
        sect6_bytes: Vec<u8>,
        sect7_bytes: Vec<u8>,
    ) -> Result<Self, GribError> {
        let mut pos = 0;
        let sect5_param = Section5Param::try_from_slice(&sect5_bytes, &mut pos)
            .map_err(|e| GribError::DecodeError(DecodeError::from(e)))?;
        let sect6_bytes = match sect6_bytes[5] {
            0x00 => sect6_bytes,
            0xff => {
                let mut sect6_bytes = sect6_bytes;
                sect6_bytes.append(&mut dummy_bitmap_for_nonnullable_data(num_points_total));
                sect6_bytes
            }
            n => {
                return Err(GribError::DecodeError(DecodeError::NotSupported(
                    "GRIB2 code table 6.0 (bit map indicator)",
                    n.into(),
                )));
            }
        };

        Ok(Self {
            num_points_total,
            sect5_param,
            sect6_bytes,
            sect7_bytes,
        })
    }

    /// Sets up a decoder for grid point values of `submessage`.
    pub fn from<R: Grib2Read>(submessage: SubMessage<R>) -> Result<Self, GribError> {
        let mut reader = submessage.9;
        let sect5 = submessage.5.body;
        let sect6 = submessage.6.body;
        let sect7 = submessage.7.body;
        let sect3_body = match submessage.3.body.body.as_ref() {
            Some(SectionBody::Section3(b3)) => b3,
            _ => return Err(GribError::InternalDataError),
        };
        let sect3_num_points = sect3_body.num_points() as usize;

        Self::new(
            sect3_num_points,
            reader.read_sect_as_slice(sect5)?,
            reader.read_sect_as_slice(sect6)?,
            reader.read_sect_as_slice(sect7)?,
        )
    }

    /// Dispatches a decoding process and gets an iterator of decoded values.
    pub fn dispatch(
        &self,
    ) -> Result<Grib2DecodedValues<'_, impl Iterator<Item = f32> + '_>, GribError> {
        let decoder = match &self.sect5_param.payload.template {
            Template::Simple(template) => {
                Grib2ValueIterator::SigSNS(simple::Simple(self, template).iter()?)
            }
            Template::Complex(template) => {
                Grib2ValueIterator::SigSC(complex::Complex(self, template).iter()?)
            }
            Template::ComplexSpatial(template) => {
                Grib2ValueIterator::SigSSCI(complex::ComplexSpatial(self, template).iter()?)
            }
            #[cfg(all(
                feature = "jpeg2000-unpack-with-openjpeg",
                feature = "jpeg2000-unpack-with-openjpeg-experimental"
            ))]
            Template::Jpeg2000(template) => {
                Grib2ValueIterator::SigSIm(jpeg2000::Jpeg2000(self, template).iter()?)
            }
            #[cfg(all(
                feature = "jpeg2000-unpack-with-openjpeg",
                not(feature = "jpeg2000-unpack-with-openjpeg-experimental")
            ))]
            Template::Jpeg2000(template) => {
                Grib2ValueIterator::SigSI(jpeg2000::Jpeg2000(self, template).iter()?)
            }
            #[cfg(feature = "png-unpack-with-png-crate")]
            Template::Png(template) => Grib2ValueIterator::SigSNV(png::Png(self, template).iter()?),
            #[cfg(feature = "ccsds-unpack-with-libaec")]
            Template::Ccsds(template) => {
                Grib2ValueIterator::SigSNV(ccsds::Ccsds(self, template).iter()?)
            }
            Template::RunLength(template) => {
                Grib2ValueIterator::SigI(run_length::RunLength(self, template).iter()?)
            }
            #[allow(unreachable_patterns)]
            _ => {
                return Err(GribError::DecodeError(DecodeError::NotSupported(
                    "GRIB2 code table 5.0 (data representation template number)",
                    self.sect5_param.payload.template_num,
                )));
            }
        };
        let decoder = BitmapDecodeIterator::new(
            self.sect6_bytes[6..].iter(),
            decoder,
            self.num_points_total,
        )?;
        Ok(Grib2DecodedValues(decoder))
    }

    pub(crate) fn num_encoded_points(&self) -> usize {
        self.sect5_param.payload.num_encoded_points as usize
    }

    pub(crate) fn sect7_payload(&self) -> &[u8] {
        &self.sect7_bytes[5..]
    }

    /// Provides access to the parameters in Section 5.
    ///
    /// # Examples
    /// ```
    /// use grib::Grib2SubmessageDecoder;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin",
    ///     )?;
    ///     let f = std::io::BufReader::new(f);
    ///     let grib2 = grib::from_reader(f)?;
    ///     let (_index, first_submessage) = grib2.iter().next().unwrap();
    ///
    ///     let decoder = Grib2SubmessageDecoder::from(first_submessage)?;
    ///     let actual = decoder.section5();
    ///     let expected = grib::def::grib2::Section5Param {
    ///         header: grib::def::grib2::SectionHeader {
    ///             len: 23,
    ///             sect_num: 5,
    ///         },
    ///         payload: grib::def::grib2::Section5Payload {
    ///             num_encoded_points: 86016,
    ///             template_num: 200,
    ///             template: grib::def::grib2::Template::RunLength(
    ///                 grib::def::grib2::RunLengthPackingTemplate {
    ///                     run_length: grib::def::grib2::RunLengthPackingParam {
    ///                         num_bits: 8,
    ///                         max_val: 3,
    ///                         max_level: 3,
    ///                         dec: 0,
    ///                         level_vals: vec![1, 2, 3],
    ///                     },
    ///                 },
    ///             ),
    ///         },
    ///     };
    ///     assert_eq!(actual, &expected);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn section5(&self) -> &Section5Param {
        &self.sect5_param
    }
}

pub struct Grib2DecodedValues<'b, I>(BitmapDecodeIterator<std::slice::Iter<'b, u8>, I>);

impl<'a, I> Iterator for Grib2DecodedValues<'a, I>
where
    I: Iterator<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self(inner) = self;
        inner.size_hint()
    }
}

enum Grib2ValueIterator<'d> {
    SigSNS(SimplePackingDecoder<NBitwiseIterator<&'d [u8]>>),
    SigSC(SimplePackingDecoder<ComplexPackingDecoded<'d>>),
    SigSSCI(
        SimplePackingDecoder<
            complex::SpatialDifferencingDecodeIterator<ComplexPackingDecoded<'d>, IntoIter<i32>>,
        >,
    ),
    #[allow(dead_code)]
    SigSI(SimplePackingDecoder<IntoIter<i32>>),
    #[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
    SigSIm(SimplePackingDecoder<self::jpeg2000::ImageIntoIter>),
    #[allow(dead_code)]
    SigSNV(SimplePackingDecoder<NBitwiseIterator<Vec<u8>>>),
    SigI(IntoIter<f32>),
}

impl<'d> Iterator for Grib2ValueIterator<'d> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::SigSNS(inner) => inner.next(),
            Self::SigSC(inner) => inner.next(),
            Self::SigSSCI(inner) => inner.next(),
            Self::SigSI(inner) => inner.next(),
            #[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
            Self::SigSIm(inner) => inner.next(),
            Self::SigSNV(inner) => inner.next(),
            Self::SigI(inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::SigSNS(inner) => inner.size_hint(),
            Self::SigSC(inner) => inner.size_hint(),
            Self::SigSSCI(inner) => inner.size_hint(),
            Self::SigSI(inner) => inner.size_hint(),
            #[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
            Self::SigSIm(inner) => inner.size_hint(),
            Self::SigSNV(inner) => inner.size_hint(),
            Self::SigI(inner) => inner.size_hint(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecodeError {
    NotSupported(&'static str, u16),
    LengthMismatch,
    UnclassifiedError(String),
}

impl From<String> for DecodeError {
    fn from(value: String) -> Self {
        Self::UnclassifiedError(value)
    }
}

impl From<&str> for DecodeError {
    fn from(value: &str) -> Self {
        Self::UnclassifiedError(value.to_owned())
    }
}

pub(crate) trait Grib2GpvUnpack {
    type Iter<'a>: Iterator<Item = f32>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError>;
}

mod bitmap;
#[cfg(feature = "ccsds-unpack-with-libaec")]
mod ccsds;
mod complex;
#[cfg(feature = "jpeg2000-unpack-with-openjpeg")]
mod jpeg2000;
#[cfg(feature = "png-unpack-with-png-crate")]
mod png;
mod run_length;
mod simple;
mod stream;
