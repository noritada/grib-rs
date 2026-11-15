use hayro_jpeg2000::{ColorSpace, DecodeSettings, DecoderContext, Image};

use crate::DecodeError;

pub(crate) type ImageIntoIter = std::vec::IntoIter<i32>;

const SIZ_HEADER_LEN: usize = 45;

pub(crate) fn decode_j2k(
    bytes: &[u8],
    expected_samples: usize,
    max_samples: Option<u32>,
    max_tiles: Option<u32>,
) -> Result<ImageIntoIter, DecodeError> {
    validate_codestream_layout(bytes, max_tiles)?;
    let image = Image::new(
        bytes,
        &DecodeSettings {
            strict: true,
            ..DecodeSettings::default()
        },
    )
    .map_err(|err| DecodeError::from(format!("parsing JPEG 2000 image failed: {err}")))?;
    if !matches!(image.color_space(), ColorSpace::Gray) || image.has_alpha() {
        return Err(DecodeError::from(
            "JPEG 2000 image must have exactly one grayscale component",
        ));
    }
    let width = usize::try_from(image.width())
        .map_err(|_| DecodeError::from("JPEG 2000 image width does not fit usize"))?;
    let height = usize::try_from(image.height())
        .map_err(|_| DecodeError::from("JPEG 2000 image height does not fit usize"))?;
    validate_sample_count(width, height, expected_samples, max_samples)?;
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

fn validate_sample_count(
    width: usize,
    height: usize,
    expected_samples: usize,
    max_samples: Option<u32>,
) -> Result<(), DecodeError> {
    let decoded_samples = width
        .checked_mul(height)
        .ok_or_else(|| DecodeError::from("JPEG 2000 image sample count overflows usize"))?;
    if let Some(max_samples) = max_samples
        && decoded_samples as u64 > u64::from(max_samples)
    {
        return Err(DecodeError::from(format!(
            "JPEG 2000 image has {decoded_samples} samples, maximum is {max_samples}",
        )));
    }
    if decoded_samples != expected_samples {
        return Err(DecodeError::from(format!(
            "JPEG 2000 image has {decoded_samples} samples, expected {expected_samples}",
        )));
    }
    Ok(())
}

fn validate_codestream_layout(bytes: &[u8], max_tiles: Option<u32>) -> Result<(), DecodeError> {
    if bytes.len() < SIZ_HEADER_LEN || bytes[..4] != [0xff, 0x4f, 0xff, 0x51] {
        return Err(DecodeError::from("invalid JPEG 2000 codestream header"));
    }

    let read_u32 = |offset| {
        u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ])
    };
    let reference_grid_width = read_u32(8);
    let reference_grid_height = read_u32(12);
    let tile_width = read_u32(24);
    let tile_height = read_u32(28);
    let tile_x_offset = read_u32(32);
    let tile_y_offset = read_u32(36);
    let component_count = u16::from_be_bytes([bytes[40], bytes[41]]);

    if component_count != 1 {
        return Err(DecodeError::from(
            "JPEG 2000 codestream must have exactly one component",
        ));
    }
    if tile_width == 0
        || tile_height == 0
        || tile_x_offset >= reference_grid_width
        || tile_y_offset >= reference_grid_height
    {
        return Err(DecodeError::from("invalid JPEG 2000 tile dimensions"));
    }

    let tile_count = (reference_grid_width - tile_x_offset)
        .div_ceil(tile_width)
        .checked_mul((reference_grid_height - tile_y_offset).div_ceil(tile_height))
        .ok_or_else(|| DecodeError::from("JPEG 2000 tile count overflows u32"))?;
    if let Some(max_tiles) = max_tiles
        && tile_count > max_tiles
    {
        return Err(DecodeError::from(format!(
            "JPEG 2000 codestream has {tile_count} tiles, maximum is {max_tiles}",
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_sample_count_is_limited() {
        assert!(validate_sample_count(4, 2, 8, Some(8)).is_ok());
        assert!(validate_sample_count(4, 2, 8, Some(7)).is_err());
        assert!(validate_sample_count(1, 1, 1, Some(0)).is_err());
        assert!(validate_sample_count(0, 1, 0, Some(0)).is_ok());
    }

    #[test]
    fn decoded_sample_count_has_no_fixed_limit() {
        let samples = 16 * 1024 * 1024 + 1;
        assert!(validate_sample_count(samples, 1, samples, None).is_ok());
        assert!(validate_sample_count(samples, 1, samples, Some(samples as u32)).is_ok());
    }

    #[test]
    fn decoded_sample_count_must_match_without_limit() {
        assert!(validate_sample_count(4, 2, 7, None).is_err());
        assert!(validate_sample_count(usize::MAX, 2, 0, None).is_err());
    }

    #[test]
    fn codestream_tile_count_is_limited() {
        let mut codestream = codestream_header();
        codestream[8..12].copy_from_slice(&4u32.to_be_bytes());
        codestream[12..16].copy_from_slice(&2u32.to_be_bytes());

        assert!(validate_codestream_layout(&codestream, Some(8)).is_ok());
        assert!(validate_codestream_layout(&codestream, Some(7)).is_err());
        assert!(validate_codestream_layout(&codestream, Some(0)).is_err());
    }

    #[test]
    fn codestream_tile_count_has_no_fixed_limit() {
        let mut codestream = codestream_header();
        codestream[8..12].copy_from_slice(&4097u32.to_be_bytes());

        assert!(validate_codestream_layout(&codestream, None).is_ok());
        assert!(validate_codestream_layout(&codestream, Some(4097)).is_ok());
    }

    #[test]
    fn codestream_tile_count_overflow_is_rejected_without_limit() {
        let mut codestream = codestream_header();
        codestream[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
        codestream[12..16].copy_from_slice(&2u32.to_be_bytes());

        assert!(validate_codestream_layout(&codestream, None).is_err());
    }

    #[test]
    fn codestream_with_multiple_components_is_rejected() {
        let mut codestream = codestream_header();
        codestream[40..42].copy_from_slice(&2u16.to_be_bytes());

        assert!(validate_codestream_layout(&codestream, None).is_err());
    }

    fn codestream_header() -> [u8; SIZ_HEADER_LEN] {
        [
            0xff, 0x4f, 0xff, 0x51, 0x00, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x01, 0x01,
        ]
    }
}
