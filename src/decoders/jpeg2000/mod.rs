use openjpeg_sys as opj;
use std::convert::TryInto;

use crate::decoders::common::*;
use crate::decoders::simple::*;
use crate::error::*;
use crate::utils::{read_as, GribInt};

mod ext;
use ext::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Jpeg2000CodeStreamDecodeError {
    NotSupported,
    DecoderSetupError,
    MainHeaderReadError,
    BodyReadError,
    LengthMismatch,
}

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIteratorWrapper<impl Iterator<Item = i32>>, GribError> {
    let sect5_data = &target.sect5_payload;
    let ref_val = read_as!(f32, sect5_data, 6);
    let exp = read_as!(u16, sect5_data, 10).as_grib_int();
    let dig = read_as!(u16, sect5_data, 12).as_grib_int();
    let nbit = read_as!(u8, sect5_data, 14);
    let value_type = read_as!(u8, sect5_data, 15);

    if nbit == 0 {
        eprintln!(
            "WARNING: nbit = 0 for JPEG 2000 code stream format decoder is not tested.
            Please report your data and help us develop the library."
        );
        let decoder = SimplePackingDecodeIteratorWrapper::FixedValue(FixedValueIterator::new(
            ref_val,
            target.num_points_encoded,
        ));
        return Ok(decoder);
    };

    if value_type != 0 {
        return Err(GribError::DecodeError(
            DecodeError::SimplePackingDecodeError(
                SimplePackingDecodeError::OriginalFieldValueTypeNotSupported,
            ),
        ));
    }

    let stream = Stream::from_bytes(&target.sect7_payload)
        .map_err(|e| GribError::DecodeError(DecodeError::Jpeg2000CodeStreamDecodeError(e)))?;
    let jp2_unpacked = decode_jp2(stream)
        .map_err(|e| GribError::DecodeError(DecodeError::Jpeg2000CodeStreamDecodeError(e)))?;
    let decoder = SimplePackingDecodeIterator::new(jp2_unpacked, ref_val, exp, dig);
    let decoder = SimplePackingDecodeIteratorWrapper::SimplePacking(decoder);
    Ok(decoder)
}

fn decode_jp2(stream: Stream) -> Result<impl Iterator<Item = i32>, Jpeg2000CodeStreamDecodeError> {
    let codec = Codec::j2k()?;

    let mut decode_params = unsafe { std::mem::zeroed::<opj::opj_dparameters>() };
    unsafe { opj::opj_set_default_decoder_parameters(&mut decode_params as *mut _) };

    if unsafe { openjpeg_sys::opj_setup_decoder(codec.0.as_ptr(), &mut decode_params) } != 1 {
        return Err(Jpeg2000CodeStreamDecodeError::DecoderSetupError);
    }

    let mut image = Image::new();

    if unsafe { opj::opj_read_header(stream.0, codec.0.as_ptr(), &mut image.0) } != 1 {
        return Err(Jpeg2000CodeStreamDecodeError::MainHeaderReadError);
    }

    if unsafe { opj::opj_decode(codec.0.as_ptr(), stream.0, image.0) } != 1 {
        return Err(Jpeg2000CodeStreamDecodeError::BodyReadError);
    }

    drop(codec);
    drop(stream);

    let width = image.width();
    let height = image.height();
    let factor = image.factor();

    let width = value_for_discard_level(width, factor);
    let height = value_for_discard_level(height, factor);

    if let [comp_gray] = image.components() {
        let vec = unsafe {
            std::slice::from_raw_parts(comp_gray.data, (width * height) as usize).to_vec()
        };
        Ok(vec.into_iter())
    } else {
        Err(Jpeg2000CodeStreamDecodeError::NotSupported)
    }
}
