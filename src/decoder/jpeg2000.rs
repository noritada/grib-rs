#[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
pub(crate) use self::image::ImageIntoIter;
use crate::{
    Grib2GpvUnpack,
    decoder::{
        DecodeError, Grib2SubmessageDecoder, jpeg2000::decoder::DecodeParams,
        param::SimplePackingParam, simple::*, stream::FixedValueIterator,
    },
};

pub(crate) struct Jpeg2000<'d>(pub(crate) &'d Grib2SubmessageDecoder);

impl<'d> Grib2GpvUnpack for Jpeg2000<'d> {
    type Iter<'a>
        = SimplePackingDecoder<Jpeg2000Iter>
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

#[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
type Jpeg2000Iter = ImageIntoIter;
#[cfg(not(feature = "jpeg2000-unpack-with-openjpeg-experimental"))]
type Jpeg2000Iter = std::vec::IntoIter<i32>;

fn decode_j2k(bytes: &[u8]) -> Result<Jpeg2000Iter, DecodeError> {
    let stream = stream::Stream::from_bytes(bytes)?;
    let decoder = decoder::Decoder::new(stream)?;
    decoder.setup(DecodeParams::default())?;
    let image = decoder.read_header()?;
    decoder.decode(&image)?;

    #[cfg(not(feature = "jpeg2000-unpack-with-openjpeg-experimental"))]
    if let [comp_gray] = image.components() {
        Ok(comp_gray.data().to_vec().into_iter())
    } else {
        Err(DecodeError::from(
            "unexpected non-gray-scale image components",
        ))
    }
    #[cfg(feature = "jpeg2000-unpack-with-openjpeg-experimental")]
    image.try_into_iter()
}

mod decoder;
mod image;
mod stream;
