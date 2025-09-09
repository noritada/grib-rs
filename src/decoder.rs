#[cfg(target_arch = "wasm32")]
use std::marker::PhantomData;

use num::ToPrimitive;

use crate::{
    context::{SectionBody, SubMessage},
    decoder::{
        bitmap::{dummy_bitmap_for_nonnullable_data, BitmapDecodeIterator},
        param::Section5Param,
        simple::SimplePackingDecodeIteratorWrapper,
    },
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
    pub(crate) sect5_bytes: Vec<u8>,
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
        let sect5_param = Section5Param::from_buf(&sect5_bytes[5..11]);
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
                )))
            }
        };

        Ok(Self {
            num_points_total,
            sect5_param,
            sect5_bytes,
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
        let decoder = match self.sect5_param.template_num {
            0 => Grib2ValueIterator::Template0(simple::decode(self)?),
            2 => Grib2ValueIterator::Template2(complex::decode_7_2(self)?),
            3 => Grib2ValueIterator::Template3(complex::decode_7_3(self)?),
            #[cfg(not(target_arch = "wasm32"))]
            40 => Grib2ValueIterator::Template40(jpeg2000::decode(self)?),
            41 => Grib2ValueIterator::Template41(png::decode(self)?),
            #[cfg(not(target_arch = "wasm32"))]
            42 => Grib2ValueIterator::Template42(ccsds::decode(self)?),
            200 => Grib2ValueIterator::Template200(run_length::decode(self)?),
            n => {
                return Err(GribError::DecodeError(DecodeError::NotSupported(
                    "GRIB2 code table 5.0 (data representation template number)",
                    n,
                )))
            }
        };
        let decoder = BitmapDecodeIterator::new(
            self.sect6_bytes[6..].iter(),
            decoder,
            self.num_points_total,
        )?;
        Ok(Grib2DecodedValues(decoder))
    }

    pub(crate) fn num_points_encoded(&self) -> usize {
        self.sect5_param.num_points_encoded as usize
    }

    pub(crate) fn sect7_payload(&self) -> &[u8] {
        &self.sect7_bytes[5..]
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

// Rust does not allow modification of generics type parameters or where clauses
// in conditonal compilation at this time. This is a trick to allow compilation
// even when JPEG 2000 code stream format support is not available (there may be
// a better way).
#[cfg(target_arch = "wasm32")]
type Grib2ValueIterator<T0, T2, T3, T41> = Grib2SubmessageDecoderIteratorWrapper<
    T0,
    T2,
    T3,
    std::vec::IntoIter<f32>,
    T41,
    std::vec::IntoIter<f32>,
>;
#[cfg(not(target_arch = "wasm32"))]
type Grib2ValueIterator<T0, T2, T3, T40, T41, T42> =
    Grib2SubmessageDecoderIteratorWrapper<T0, T2, T3, T40, T41, T42>;

enum Grib2SubmessageDecoderIteratorWrapper<T0, T2, T3, T40, T41, T42> {
    Template0(SimplePackingDecodeIteratorWrapper<T0>),
    Template2(SimplePackingDecodeIteratorWrapper<T2>),
    Template3(SimplePackingDecodeIteratorWrapper<T3>),
    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    Template40(PhantomData<T40>),
    #[cfg(not(target_arch = "wasm32"))]
    Template40(SimplePackingDecodeIteratorWrapper<T40>),
    Template41(SimplePackingDecodeIteratorWrapper<T41>),
    #[allow(dead_code)]
    #[cfg(target_arch = "wasm32")]
    Template42(PhantomData<T42>),
    #[cfg(not(target_arch = "wasm32"))]
    Template42(SimplePackingDecodeIteratorWrapper<T42>),
    Template200(std::vec::IntoIter<f32>),
}

impl<T0, T2, T3, T40, T41, T42> Iterator
    for Grib2SubmessageDecoderIteratorWrapper<T0, T2, T3, T40, T41, T42>
where
    T0: Iterator,
    <T0 as Iterator>::Item: ToPrimitive,
    T2: Iterator,
    <T2 as Iterator>::Item: ToPrimitive,
    T3: Iterator,
    <T3 as Iterator>::Item: ToPrimitive,
    T40: Iterator,
    <T40 as Iterator>::Item: ToPrimitive,
    T41: Iterator,
    <T41 as Iterator>::Item: ToPrimitive,
    T42: Iterator,
    <T42 as Iterator>::Item: ToPrimitive,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Template0(inner) => inner.next(),
            Self::Template2(inner) => inner.next(),
            Self::Template3(inner) => inner.next(),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Template40(inner) => inner.next(),
            #[cfg(target_arch = "wasm32")]
            Self::Template40(_) => unreachable!(),
            Self::Template41(inner) => inner.next(),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Template42(inner) => inner.next(),
            #[cfg(target_arch = "wasm32")]
            Self::Template42(_) => unreachable!(),
            Self::Template200(inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Template0(inner) => inner.size_hint(),
            Self::Template2(inner) => inner.size_hint(),
            Self::Template3(inner) => inner.size_hint(),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Template40(inner) => inner.size_hint(),
            #[cfg(target_arch = "wasm32")]
            Self::Template40(_) => unreachable!(),
            Self::Template41(inner) => inner.size_hint(),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Template42(inner) => inner.size_hint(),
            #[cfg(target_arch = "wasm32")]
            Self::Template42(_) => unreachable!(),
            Self::Template200(inner) => inner.size_hint(),
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
#[cfg(not(target_arch = "wasm32"))]
mod ccsds;
mod complex;
#[cfg(not(target_arch = "wasm32"))]
mod jpeg2000;
mod param;
mod png;
mod run_length;
mod simple;
mod stream;
