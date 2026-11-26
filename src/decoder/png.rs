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

        let buf = read_image_buffer(target.sect7_payload(), template.simple.num_bits)
            .map_err(|e| DecodeError::from(format!("PNG decode error: {e}")))?;
        let iter = NBitwiseIterator::new(buf, usize::from(template.simple.num_bits));
        let iter = NonZeroSimplePackingDecoder::new(iter, &template.simple);
        let iter = SimplePackingDecoder::NonZeroLength(iter);
        Ok(iter)
    }
}

fn read_image_buffer(buf: &[u8], num_bits: u8) -> Result<Vec<u8>, String> {
    let reader = std::io::Cursor::new(&buf);
    let decoder = png::Decoder::new(reader);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let buf_size = reader
        .output_buffer_size()
        .ok_or("Getting output buffer size failed")?;
    let mut out_buf = vec![0; buf_size];
    let info = reader.next_frame(&mut out_buf).map_err(|e| e.to_string())?;
    if info.height > 1 && matches!(num_bits, 1 | 2 | 4) && info.bit_depth as u8 == num_bits {
        let bits_per_pixel = usize::from(num_bits);
        let pixels_per_byte = 8 / bits_per_pixel;
        let last_byte_bits = (info.width as usize % pixels_per_byte) * bits_per_pixel;
        if last_byte_bits != 0 {
            // Pack the samples contiguously for the bit iterator.
            let len = remove_scanline_padding(
                &mut out_buf[..info.buffer_size()],
                info.line_size,
                last_byte_bits,
            );
            out_buf.truncate(len);
        }
    }
    Ok(out_buf)
}

fn remove_scanline_padding(buf: &mut [u8], line_size: usize, last_byte_bits: usize) -> usize {
    let mut write_pos = 0;
    let mut bytes_left_in_row = line_size;
    let mut pending = 0u16;
    let mut pending_bits = 0;
    for read_pos in 0..buf.len() {
        bytes_left_in_row -= 1;
        let bits = if bytes_left_in_row == 0 {
            bytes_left_in_row = line_size;
            last_byte_bits
        } else {
            8
        };

        pending = (pending << bits) | u16::from(buf[read_pos] >> (8 - bits));
        pending_bits += bits;
        if pending_bits >= 8 {
            pending_bits -= 8;
            buf[write_pos] = (pending >> pending_bits) as u8;
            write_pos += 1;
            pending &= (1 << pending_bits) - 1;
        }
    }
    if pending_bits != 0 {
        buf[write_pos] = (pending << (8 - pending_bits)) as u8;
        write_pos += 1;
    }
    write_pos
}

#[cfg(test)]
mod tests {
    use crate::{DecodeError, Grib2SubmessageDecoder, GribError};

    fn encode_png(
        width: u32,
        height: u32,
        color: png::ColorType,
        depth: png::BitDepth,
        pixels: &[u8],
    ) -> Result<Vec<u8>, png::EncodingError> {
        let mut image = Vec::new();
        let mut encoder = png::Encoder::new(&mut image, width, height);
        encoder.set_color(color);
        encoder.set_depth(depth);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(pixels)?;
        writer.finish()?;
        Ok(image)
    }

    fn png_decoder(
        num_points: u8,
        num_bits: u8,
        image: &[u8],
    ) -> Result<Grib2SubmessageDecoder, GribError> {
        let mut sect5 = vec![0, 0, 0, 21, 5, 0, 0, 0, num_points, 0, 41];
        sect5.extend_from_slice(&[0; 10]); // Zero reference and scales, float values.
        sect5[19] = num_bits;
        let sect6 = vec![0, 0, 0, 6, 6, 255]; // No bitmap.
        let mut sect7 = ((5 + image.len()) as u32).to_be_bytes().to_vec();
        sect7.push(7);
        sect7.extend_from_slice(image);
        Grib2SubmessageDecoder::new(usize::from(num_points), sect5, sect6, sect7)
    }

    fn assert_png_decode_error(image: &[u8]) -> Result<(), GribError> {
        let decoder = png_decoder(1, 8, image)?;
        assert!(matches!(
            decoder.dispatch(),
            Err(GribError::DecodeError(DecodeError::UnclassifiedError(message)))
                if message.starts_with("PNG decode error:")
        ));
        Ok(())
    }

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

    #[test]
    fn decode_png_rejects_invalid_header() -> Result<(), GribError> {
        assert_png_decode_error(b"not a PNG image")
    }

    #[test]
    fn decode_png_rejects_truncated_image() -> Result<(), Box<dyn std::error::Error>> {
        let mut image = encode_png(1, 1, png::ColorType::Grayscale, png::BitDepth::Eight, &[42])?;
        let idat = image
            .windows(4)
            .position(|bytes| bytes == b"IDAT")
            .ok_or("generated image has no IDAT chunk")?;
        // Keep the chunk header so the decoder reaches the frame data.
        image.truncate(idat + 4);

        assert_png_decode_error(&image)?;
        Ok(())
    }

    #[test]
    fn decode_png_accepts_subbyte_grayscale_depths() -> Result<(), Box<dyn std::error::Error>> {
        let cases: [(png::BitDepth, u8, &[f32]); 3] = [
            (
                png::BitDepth::One,
                0b1010_0110,
                &[1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
            ),
            (png::BitDepth::Two, 0b00_01_10_11, &[0.0, 1.0, 2.0, 3.0]),
            (png::BitDepth::Four, 0b0011_1010, &[3.0, 10.0]),
        ];
        for (depth, packed, expected) in cases {
            let num_points = expected.len() as u8;
            let image = encode_png(
                u32::from(num_points),
                1,
                png::ColorType::Grayscale,
                depth,
                &[packed],
            )?;
            let decoder = png_decoder(num_points, depth as u8, &image)?;
            let actual = decoder.dispatch()?.collect::<Vec<_>>();

            assert_eq!(actual.as_slice(), expected);
        }
        Ok(())
    }

    #[test]
    fn decode_png_skips_scanline_padding() -> Result<(), Box<dyn std::error::Error>> {
        let cases: [(png::BitDepth, &[u8], [f32; 6]); 3] = [
            (
                png::BitDepth::One,
                &[0b1010_0000, 0b0110_0000],
                [1.0, 0.0, 1.0, 0.0, 1.0, 1.0],
            ),
            (
                png::BitDepth::Two,
                &[0b00_01_10_00, 0b11_10_01_00],
                [0.0, 1.0, 2.0, 3.0, 2.0, 1.0],
            ),
            (
                png::BitDepth::Four,
                &[0x12, 0x30, 0x45, 0x60],
                [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            ),
        ];
        for (depth, pixels, expected) in cases {
            let image = encode_png(3, 2, png::ColorType::Grayscale, depth, pixels)?;
            let decoder = png_decoder(expected.len() as u8, depth as u8, &image)?;
            let actual = decoder.dispatch()?.collect::<Vec<_>>();

            assert_eq!(actual, expected, "{depth:?} scanlines");
        }
        Ok(())
    }

    #[test]
    fn decode_png_accepts_32_bit_rgba() -> Result<(), Box<dyn std::error::Error>> {
        let image = encode_png(
            1,
            1,
            png::ColorType::Rgba,
            png::BitDepth::Eight,
            &[0, 1, 2, 3],
        )?;
        let decoder = png_decoder(1, 32, &image)?;

        let actual = decoder.dispatch()?.collect::<Vec<_>>();

        assert_eq!(actual, [66_051.0]); // 0x00010203, including the alpha byte.
        Ok(())
    }
}
