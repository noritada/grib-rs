use crate::context::{SectionBody, SubMessage};
use crate::decoders::bitmap::{create_bitmap_for_nonnullable_data, BitmapDecodeIterator};
use crate::decoders::complex::*;
use crate::decoders::jpeg2000::*;
use crate::decoders::run_length::*;
use crate::decoders::simple::*;
use crate::error::*;
use crate::reader::Grib2Read;

pub(crate) struct Grib2SubmessageEncoded {
    pub(crate) num_points_encoded: usize,
    pub(crate) sect5_payload: Box<[u8]>,
    pub(crate) sect7_payload: Box<[u8]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecodeError {
    TemplateNumberUnsupported,
    BitMapIndicatorUnsupported,
    SimplePackingDecodeError(SimplePackingDecodeError),
    ComplexPackingDecodeError(ComplexPackingDecodeError),
    Jpeg2000CodeStreamDecodeError(Jpeg2000CodeStreamDecodeError),
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

pub fn dispatch<R: Grib2Read>(submessage: SubMessage<R>) -> Result<Box<[f32]>, GribError> {
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
    let encoded = Grib2SubmessageEncoded {
        num_points_encoded: sect5_body.num_points() as usize,
        sect5_payload: reader.read_sect_payload_as_slice(sect5)?,
        sect7_payload: reader.read_sect_payload_as_slice(sect7)?,
    };

    let num_points_total = sect3_num_points;
    let decoded = match sect5_body.repr_tmpl_num() {
        0 => {
            let decoder = SimplePackingDecoder::decode(encoded)?;
            let decoder = BitmapDecodeIterator::new(bitmap.iter(), decoder, num_points_total)?;
            decoder.collect::<Vec<_>>()
        }
        3 => {
            let decoder = ComplexPackingDecoder::decode(encoded)?;
            let decoder = BitmapDecodeIterator::new(bitmap.iter(), decoder, num_points_total)?;
            decoder.collect::<Vec<_>>()
        }
        40 => {
            let decoder = Jpeg2000CodeStreamDecoder::decode(encoded)?;
            let decoder = BitmapDecodeIterator::new(bitmap.iter(), decoder, num_points_total)?;
            decoder.collect::<Vec<_>>()
        }
        200 => {
            let decoder = RunLengthEncodingDecoder::decode(encoded)?;
            let decoder = BitmapDecodeIterator::new(bitmap.iter(), decoder, num_points_total)?;
            decoder.collect::<Vec<_>>()
        }
        _ => {
            return Err(GribError::DecodeError(
                DecodeError::TemplateNumberUnsupported,
            ))
        }
    };
    Ok(decoded.into_boxed_slice())
}
