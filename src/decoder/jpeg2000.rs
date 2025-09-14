use std::vec::IntoIter;

use openjpeg_sys as opj;

use crate::{
    decoder::{
        param::SimplePackingParam, simple::*, stream::FixedValueIterator, DecodeError,
        Grib2SubmessageDecoder,
    },
    Grib2GpvUnpack,
};

mod ext;
use ext::*;

pub(crate) struct Jpeg2000<'d>(pub(crate) &'d Grib2SubmessageDecoder);

impl<'d> Grib2GpvUnpack for Jpeg2000<'d> {
    type Iter<'a>
        = SimplePackingDecoder<IntoIter<i32>>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError> {
        let Self(target) = self;
        let sect5_data = &target.sect5_bytes;
        let simple_param = SimplePackingParam::from_buf(&sect5_data[11..21])?;

        if simple_param.nbit == 0 {
            // Tested with the World Aviation Forecast System (WAFS) GRIV files from the repo: https://aviationweather.gov/wifs/api.html
            // See #111 and #113.
            let decoder = SimplePackingDecoder::ZeroLength(FixedValueIterator::new(
                simple_param.zero_bit_reference_value(),
                target.num_points_encoded(),
            ));
            return Ok(decoder);
        };

        let stream = Stream::from_bytes(target.sect7_payload())?;
        let jp2_unpacked = decode_jp2(stream)?;
        let decoder = NonZeroSimplePackingDecoder::new(jp2_unpacked, &simple_param);
        let decoder = SimplePackingDecoder::NonZeroLength(decoder);
        Ok(decoder)
    }
}

fn decode_jp2(stream: Stream) -> Result<IntoIter<i32>, DecodeError> {
    let codec = Codec::j2k()?;

    let mut decode_params = unsafe { std::mem::zeroed::<opj::opj_dparameters>() };
    unsafe { opj::opj_set_default_decoder_parameters(&mut decode_params as *mut _) };

    if unsafe { openjpeg_sys::opj_setup_decoder(codec.0.as_ptr(), &mut decode_params) } != 1 {
        return Err(DecodeError::from("setup of openjpeg decoder failed"));
    }

    let mut image = Image::new();

    if unsafe { opj::opj_read_header(stream.0, codec.0.as_ptr(), &mut image.0) } != 1 {
        return Err(DecodeError::from(
            "decoding of JPEG 2000 image header failed",
        ));
    }

    if unsafe { opj::opj_decode(codec.0.as_ptr(), stream.0, image.0) } != 1 {
        return Err(DecodeError::from("decoding of JPEG 2000 image failed"));
    }

    drop(codec);
    drop(stream);

    if let [comp_gray] = image.components() {
        let len = (comp_gray.w * comp_gray.h) as usize;
        let vec = unsafe { std::slice::from_raw_parts(comp_gray.data, len).to_vec() };
        Ok(vec.into_iter())
    } else {
        Err(DecodeError::from(
            "unexpected non-gray-scale image components",
        ))
    }
}

mod decoder;
mod image;
mod stream;
