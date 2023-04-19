use num::ToPrimitive;

use crate::{
    context::{SectionBody, SubMessage},
    decoders::{
        bitmap::{create_bitmap_for_nonnullable_data, BitmapDecodeIterator},
        complex::{self, ComplexPackingDecodeError},
        jpeg2000::{self, Jpeg2000CodeStreamDecodeError},
        png::{self, PngDecodeError},
        run_length::{self, RunLengthEncodingDecodeError},
        simple::{
            self, SimplePackingDecodeError, SimplePackingDecodeIterator,
            SimplePackingDecodeIteratorWrapper,
        },
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
///     let f =
///         std::fs::File::open("testdata/CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2")?;
///     let f = std::io::BufReader::new(f);
///     let grib2 = grib::from_reader(f)?;
///     let (_index, first_submessage) = grib2.iter().next().unwrap();
///
///     let decoder = Grib2SubmessageDecoder::from(first_submessage)?;
///     let mut decoded = decoder.dispatch()?;
///     assert_eq!(decoded.size_hint(), (1126500, Some(1126500)));
///
///     let first_value = decoded.next();
///     assert_eq!(first_value.map(|f| f.round()), Some(236.0_f32));
///
///     let last_value = decoded.nth(1126498);
///     assert_eq!(last_value.map(|f| f.round()), Some(286.0_f32));
///
///     let next_to_last_value = decoded.next();
///     assert_eq!(next_to_last_value, None);
///     Ok(())
/// }
/// ```
pub struct Grib2SubmessageDecoder {
    num_points_total: usize,
    pub(crate) num_points_encoded: usize,
    template_num: u16,
    pub(crate) sect5_payload: Box<[u8]>,
    bitmap: Vec<u8>,
    pub(crate) sect7_payload: Box<[u8]>,
}

impl Grib2SubmessageDecoder {
    fn new(
        num_points_total: usize,
        num_points_encoded: usize,
        template_num: u16,
        sect5_payload: Box<[u8]>,
        bitmap: Vec<u8>,
        sect7_payload: Box<[u8]>,
    ) -> Self {
        Self {
            num_points_total,
            num_points_encoded,
            template_num,
            sect5_payload,
            bitmap,
            sect7_payload,
        }
    }

    /// Sets up a decoder for grid point values of `submessage`.
    pub fn from<R: Grib2Read>(submessage: SubMessage<R>) -> Result<Self, GribError> {
        let mut reader = submessage.9;
        let sect5 = submessage.5.body;
        let sect6 = submessage.6.body;
        let sect7 = submessage.7.body;
        let (sect3_body, sect5_body, sect6_body) = match (
            submessage.3.body.body.as_ref(),
            sect5.body.as_ref(),
            sect6.body.as_ref(),
        ) {
            (
                Some(SectionBody::Section3(b3)),
                Some(SectionBody::Section5(b5)),
                Some(SectionBody::Section6(b6)),
            ) => (b3, b5, b6),
            _ => return Err(GribError::InternalDataError),
        };
        let sect3_num_points = sect3_body.num_points() as usize;

        let bitmap = match sect6_body.bitmap_indicator {
            0x00 => {
                let sect6_data = reader.read_sect_payload_as_slice(sect6)?;
                sect6_data[1..].into()
            }
            0xff => {
                let num_points = sect3_num_points;
                create_bitmap_for_nonnullable_data(num_points)
            }
            _ => {
                return Err(GribError::DecodeError(
                    DecodeError::BitMapIndicatorUnsupported,
                ));
            }
        };

        Ok(Self::new(
            sect3_num_points,
            sect5_body.num_points() as usize,
            sect5_body.repr_tmpl_num(),
            reader.read_sect_payload_as_slice(sect5)?,
            bitmap,
            reader.read_sect_payload_as_slice(sect7)?,
        ))
    }

    /// Dispatches a decoding process and gets an iterator of decoded values.
    pub fn dispatch(
        &self,
    ) -> Result<Grib2DecodedValues<impl Iterator<Item = f32> + '_>, GribError> {
        let decoder = match self.template_num {
            0 => Grib2SubmessageDecoderIteratorWrapper::Template0(simple::decode(self)?),
            3 => Grib2SubmessageDecoderIteratorWrapper::Template3(complex::decode(self)?),
            40 => Grib2SubmessageDecoderIteratorWrapper::Template40(jpeg2000::decode(self)?),
            41 => Grib2SubmessageDecoderIteratorWrapper::Template41(png::decode(self)?),
            200 => Grib2SubmessageDecoderIteratorWrapper::Template200(run_length::decode(self)?),
            _ => {
                return Err(GribError::DecodeError(
                    DecodeError::TemplateNumberUnsupported,
                ))
            }
        };
        let decoder =
            BitmapDecodeIterator::new(self.bitmap.iter(), decoder, self.num_points_total)?;
        Ok(Grib2DecodedValues(decoder))
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

enum Grib2SubmessageDecoderIteratorWrapper<I, J, K, L> {
    Template0(SimplePackingDecodeIteratorWrapper<I>),
    Template3(SimplePackingDecodeIteratorWrapper<J>),
    Template40(SimplePackingDecodeIteratorWrapper<K>),
    Template41(SimplePackingDecodeIterator<L>),
    Template200(std::vec::IntoIter<f32>),
}

impl<I, J, K, L> Iterator for Grib2SubmessageDecoderIteratorWrapper<I, J, K, L>
where
    I: Iterator,
    <I as Iterator>::Item: ToPrimitive,
    J: Iterator,
    <J as Iterator>::Item: ToPrimitive,
    K: Iterator,
    <K as Iterator>::Item: ToPrimitive,
    L: Iterator,
    <L as Iterator>::Item: ToPrimitive,
{
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Template0(inner) => inner.next(),
            Self::Template3(inner) => inner.next(),
            Self::Template40(inner) => inner.next(),
            Self::Template41(inner) => inner.next(),
            Self::Template200(inner) => inner.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Template0(inner) => inner.size_hint(),
            Self::Template3(inner) => inner.size_hint(),
            Self::Template40(inner) => inner.size_hint(),
            Self::Template41(inner) => inner.size_hint(),
            Self::Template200(inner) => inner.size_hint(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecodeError {
    TemplateNumberUnsupported,
    BitMapIndicatorUnsupported,
    SimplePackingDecodeError(SimplePackingDecodeError),
    ComplexPackingDecodeError(ComplexPackingDecodeError),
    Jpeg2000CodeStreamDecodeError(Jpeg2000CodeStreamDecodeError),
    PngDecodeError(PngDecodeError),
    RunLengthEncodingDecodeError(RunLengthEncodingDecodeError),
    LengthMismatch,
}

impl From<SimplePackingDecodeError> for DecodeError {
    fn from(e: SimplePackingDecodeError) -> Self {
        Self::SimplePackingDecodeError(e)
    }
}

impl From<ComplexPackingDecodeError> for DecodeError {
    fn from(e: ComplexPackingDecodeError) -> Self {
        Self::ComplexPackingDecodeError(e)
    }
}

impl From<Jpeg2000CodeStreamDecodeError> for DecodeError {
    fn from(e: Jpeg2000CodeStreamDecodeError) -> Self {
        Self::Jpeg2000CodeStreamDecodeError(e)
    }
}

impl From<RunLengthEncodingDecodeError> for DecodeError {
    fn from(e: RunLengthEncodingDecodeError) -> Self {
        Self::RunLengthEncodingDecodeError(e)
    }
}
