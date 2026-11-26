use std::sync::Once;

use crate::{
    DecodeError, Grib2GpvUnpack, Grib2SubmessageDecoder,
    decoder::{
        simple::{NonZeroSimplePackingDecoder, SimplePackingDecoder},
        stream::{FixedValueIterator, NBitwiseIterator},
    },
};

static PNG_NUM_BITS_ZERO_WARNING_ONCE: Once = Once::new();

pub(crate) struct Png<'d>(
    pub(crate) &'d Grib2SubmessageDecoder,
    pub(crate) &'d crate::def::grib2::template::Template5_41,
);

impl<'d> Grib2GpvUnpack for Png<'d> {
    type Iter<'a>
        = SimplePackingDecoder<NBitwiseIterator<Vec<u8>>>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError> {
        let Self(target, template) = self;
        super::orig_field_type_is_supported(template.orig_field_type.0)?;

        if template.simple.num_bits == 0 {
            PNG_NUM_BITS_ZERO_WARNING_ONCE.call_once(|| {
                eprintln!(
                    "WARNING: data with num_bits being 0 for PNG decoder is not tested.
                    Please report your data and help us develop the library."
                );
            });
            let decoder = SimplePackingDecoder::ZeroLength(FixedValueIterator::new(
                template.simple.zero_bit_reference_value(),
                target.num_encoded_points(),
            ));
            return Ok(decoder);
        };

        // Valid values for `num_bits` can be:
        //
        // - 1, 2, 4, 8, or 16 : Treat as greyscale image
        // - 24 : Treat as RGB colour image (each component having 8-bit depth)
        // - 32 : Treat as RGB w/ alpha sample colour image (each component
        //   having 8-bit depth)
        //
        // Since we have verified the decoding process through testing for cases
        // where `num_bits` is 8, 16, and 24, we believe other cases
        // should also be fine.

        let buf = read_image_buffer(target.sect7_payload())
            .map_err(|e| DecodeError::from(format!("PNG decode error: {e}")))?;
        let iter = NBitwiseIterator::new(buf, usize::from(template.simple.num_bits));
        let iter = NonZeroSimplePackingDecoder::new(iter, &template.simple);
        let iter = SimplePackingDecoder::NonZeroLength(iter);
        Ok(iter)
    }
}

fn read_image_buffer(buf: &[u8]) -> Result<Vec<u8>, String> {
    let reader = std::io::Cursor::new(&buf);
    let decoder = png::Decoder::new(reader);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let buf_size = reader
        .output_buffer_size()
        .ok_or("Getting output buffer size failed")?;
    let mut out_buf = vec![0; buf_size];
    let _info = reader.next_frame(&mut out_buf).map_err(|e| e.to_string())?;
    Ok(out_buf)
}

#[cfg(test)]
mod tests {
    use super::Grib2SubmessageDecoder;

    #[test]
    fn decode_png_constant_field_without_image() -> Result<(), Box<dyn std::error::Error>> {
        let mut sect5 = vec![0, 0, 0, 21, 5, 0, 0, 0, 3, 0, 41]; // Three points, template 5.41.
        sect5.extend_from_slice(&25.0_f32.to_be_bytes());
        sect5.extend_from_slice(&[0, 0, 0, 1, 0, 0]); // E = 0, D = 1, zero bits, float.
        let sect6 = vec![0, 0, 0, 6, 6, 255]; // No bitmap.
        let sect7 = vec![0, 0, 0, 5, 7]; // Header only, no PNG image.
        let decoder = Grib2SubmessageDecoder::new(3, sect5, sect6, sect7)?;

        let actual = decoder.dispatch()?.collect::<Vec<_>>();

        assert_eq!(actual, [2.5; 3]);
        Ok(())
    }
}
