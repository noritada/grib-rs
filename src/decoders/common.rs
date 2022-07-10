use std::cell::RefMut;

use crate::context::{SectionBody, SectionInfo};
use crate::decoders::bitmap::create_bitmap_for_nonnullable_data;
use crate::decoders::complex::*;
use crate::decoders::jpeg2000::*;
use crate::decoders::run_length::*;
use crate::decoders::simple::*;
use crate::error::*;
use crate::reader::Grib2Read;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DecodeError {
    TemplateNumberUnsupported,
    BitMapIndicatorUnsupported,
    SimplePackingDecodeError(SimplePackingDecodeError),
    ComplexPackingDecodeError(ComplexPackingDecodeError),
    Jpeg2000CodeStreamDecodeError(Jpeg2000CodeStreamDecodeError),
    RunLengthEncodingDecodeError(RunLengthEncodingDecodeError),
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

pub fn dispatch<R: Grib2Read>(
    sect3: &SectionInfo,
    sect5: &SectionInfo,
    sect6: &SectionInfo,
    sect7: &SectionInfo,
    mut reader: RefMut<R>,
) -> Result<Box<[f32]>, GribError> {
    let (sect3_body, sect5_body, sect6_body) = match (&sect3.body, &sect5.body, &sect6.body) {
        (
            Some(SectionBody::Section3(b3)),
            Some(SectionBody::Section5(b5)),
            Some(SectionBody::Section6(b6)),
        ) => (b3, b5, b6),
        _ => return Err(GribError::InternalDataError),
    };
    let sect3_num_points = sect3_body.num_points as usize;

    let bitmap = match sect6_body.bitmap_indicator {
        0x00 => {
            let sect6_data = reader.read_sect_body_bytes(sect6)?;
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
    let decoded = match sect5_body.repr_tmpl_num {
        0 => SimplePackingDecoder::decode(sect3_num_points, sect5, bitmap, sect7, reader)?,
        3 => ComplexPackingDecoder::decode(sect3_num_points, sect5, bitmap, sect7, reader)?,
        40 => Jpeg2000CodeStreamDecoder::decode(sect3_num_points, sect5, bitmap, sect7, reader)?,
        200 => RunLengthEncodingDecoder::decode(sect3_num_points, sect5, bitmap, sect7, reader)?,
        _ => {
            return Err(GribError::DecodeError(
                DecodeError::TemplateNumberUnsupported,
            ))
        }
    };
    Ok(decoded)
}

pub(crate) trait Grib2DataDecode<R> {
    fn decode(
        sect3_num_points: usize,
        sect5: &SectionInfo,
        bitmap: Vec<u8>,
        sect7: &SectionInfo,
        reader: RefMut<R>,
    ) -> Result<Box<[f32]>, GribError>;
}
