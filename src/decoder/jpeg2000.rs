use std::vec::IntoIter;

use crate::{
    decoder::{
        jpeg2000::decoder::DecodeParams, param::SimplePackingParam, simple::*,
        stream::FixedValueIterator, DecodeError, Grib2SubmessageDecoder,
    },
    Grib2GpvUnpack,
};

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

        let jp2_unpacked = decode_j2k(target.sect7_payload())?;
        let decoder = NonZeroSimplePackingDecoder::new(jp2_unpacked, &simple_param);
        let decoder = SimplePackingDecoder::NonZeroLength(decoder);
        Ok(decoder)
    }
}

fn decode_j2k(bytes: &[u8]) -> Result<IntoIter<i32>, DecodeError> {
    let stream = stream::Stream::from_bytes(bytes)?;
    let decoder = decoder::Decoder::new(stream)?;
    decoder.setup(DecodeParams::default())?;
    let image = decoder.read_header()?;
    decoder.decode(&image)?;

    if let [comp_gray] = image.components() {
        Ok(comp_gray.data().to_vec().into_iter())
    } else {
        Err(DecodeError::from(
            "unexpected non-gray-scale image components",
        ))
    }
}

mod decoder;
mod image;
mod stream;
