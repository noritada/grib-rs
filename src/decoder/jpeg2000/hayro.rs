use hayro_jpeg2000::{DecodeSettings, DecoderContext, Image};

use crate::DecodeError;

pub(crate) type ImageIntoIter = std::vec::IntoIter<i32>;

pub(crate) fn decode_j2k(bytes: &[u8]) -> Result<ImageIntoIter, DecodeError> {
    let image = Image::new(bytes, &DecodeSettings::default())
        .map_err(|err| DecodeError::from(format!("parsing JPEG 2000 image failed: {err}")))?;
    let mut context = DecoderContext::default();
    let decoded = image
        .decode(&mut context)
        .map_err(|err| DecodeError::from(format!("decoding JPEG 2000 image failed: {err}")))?;
    let [component] = decoded.components() else {
        return Err(DecodeError::from(
            "unexpected non-gray-scale image components",
        ));
    };

    let samples = component
        .samples()
        .iter()
        .map(|&sample| {
            if !sample.is_finite()
                || sample.fract() != 0.0
                || sample < i32::MIN as f32
                || sample >= i32::MAX as f32
            {
                return Err(DecodeError::from(
                    "JPEG 2000 component contains an invalid integer sample",
                ));
            }
            Ok(sample as i32)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(samples.into_iter())
}
