use crate::{
    Grib2GpvUnpack,
    decoder::{DecodeError, Grib2SubmessageDecoder, simple::*, stream::FixedValueIterator},
};

pub(crate) struct Jpeg2000<'d>(
    pub(crate) &'d Grib2SubmessageDecoder,
    pub(crate) &'d crate::def::grib2::template::Template5_40,
);

impl<'d> Grib2GpvUnpack for Jpeg2000<'d> {
    type Iter<'a>
        = SimplePackingDecoder<ImageIntoIter>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError> {
        let Self(target, template) = self;
        super::orig_field_type_is_supported(template.orig_field_type)?;

        if template.simple.num_bits == 0 {
            // Tested with the World Aviation Forecast System (WAFS) GRIV files from the repo: https://aviationweather.gov/wifs/api.html
            // See #111 and #113.
            let decoder = SimplePackingDecoder::ZeroLength(FixedValueIterator::new(
                template.simple.zero_bit_reference_value(),
                target.num_encoded_points(),
            ));
            return Ok(decoder);
        };

        let unpacked = decode_j2k(target.sect7_payload())?;
        let decoder = NonZeroSimplePackingDecoder::new(unpacked, &template.simple);
        Ok(SimplePackingDecoder::NonZeroLength(decoder))
    }
}

#[cfg(feature = "jpeg2000-unpack-with-hayro")]
pub(crate) type ImageIntoIter = std::vec::IntoIter<i32>;
#[cfg(all(
    not(feature = "jpeg2000-unpack-with-hayro"),
    feature = "jpeg2000-unpack-with-openjpeg"
))]
pub(crate) type ImageIntoIter = openjpeg::ImageIntoIter;

#[cfg(feature = "jpeg2000-unpack-with-hayro")]
fn decode_j2k(bytes: &[u8]) -> Result<ImageIntoIter, DecodeError> {
    hayro::decode_j2k(bytes)
}

#[cfg(all(
    not(feature = "jpeg2000-unpack-with-hayro"),
    feature = "jpeg2000-unpack-with-openjpeg"
))]
fn decode_j2k(bytes: &[u8]) -> Result<ImageIntoIter, DecodeError> {
    openjpeg::decode_j2k(bytes)
}

#[cfg(test)]
mod tests {
    #[cfg(all(
        feature = "jpeg2000-unpack-with-hayro",
        feature = "jpeg2000-unpack-with-openjpeg"
    ))]
    use std::{fs::File, io::BufReader};

    #[allow(dead_code)]
    use super::*;

    #[test]
    #[cfg(all(
        feature = "jpeg2000-unpack-with-hayro",
        feature = "jpeg2000-unpack-with-openjpeg"
    ))]
    fn hayro_has_priority_and_matches_openjpeg() -> Result<(), Box<dyn std::error::Error>> {
        let f = File::open(crate::test_utils::data::grib2::CMC_GLB)?;
        let grib2 = crate::from_reader(BufReader::new(f))?;
        let (_index, submessage) = grib2.iter().next().ok_or("GRIB file has no submessages")?;
        let decoder = crate::Grib2SubmessageDecoder::from(submessage)?;
        let payload = decoder.sect7_payload();

        let openjpeg = openjpeg::decode_j2k(payload)
            .map_err(|err| format!("{err:?}"))?
            .collect::<Vec<_>>();
        let hayro = hayro::decode_j2k(payload)
            .map_err(|err| format!("{err:?}"))?
            .collect::<Vec<_>>();
        let selected: hayro::ImageIntoIter =
            decode_j2k(payload).map_err(|err| format!("{err:?}"))?;

        assert_eq!(hayro, openjpeg);
        assert_eq!(selected.collect::<Vec<_>>(), hayro);
        Ok(())
    }
}

#[cfg(feature = "jpeg2000-unpack-with-hayro")]
mod hayro;
#[cfg(all(
    feature = "jpeg2000-unpack-with-openjpeg",
    any(not(feature = "jpeg2000-unpack-with-hayro"), test)
))]
mod openjpeg;
