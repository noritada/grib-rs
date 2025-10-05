use crate::{
    DecodeError, Grib2GpvUnpack, Grib2SubmessageDecoder,
    decoder::{
        simple::{NonZeroSimplePackingDecoder, SimplePackingDecoder},
        stream::{FixedValueIterator, NBitwiseIterator},
    },
};

pub(crate) struct Png<'d>(
    pub(crate) &'d Grib2SubmessageDecoder,
    pub(crate) &'d crate::def::grib2::PngTemplate,
);

impl<'d> Grib2GpvUnpack for Png<'d> {
    type Iter<'a>
        = SimplePackingDecoder<NBitwiseIterator<Vec<u8>>>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError> {
        let Self(target, template) = self;
        super::orig_field_type_is_supported(template.orig_field_type)?;

        let buf = read_image_buffer(target.sect7_payload())
            .map_err(|e| DecodeError::from(format!("PNG decode error: {e}")))?;

        if template.simple.num_bits == 0 {
            eprintln!(
                "WARNING: nbit = 0 for PNG decoder is not tested.
                Please report your data and help us develop the library."
            );
            let decoder = SimplePackingDecoder::ZeroLength(FixedValueIterator::new(
                template.simple.zero_bit_reference_value(),
                target.num_encoded_points(),
            ));
            return Ok(decoder);
        };

        if template.simple.num_bits != 16 {
            eprintln!(
                "WARNING: nbit != 16 for PNG decoder is not tested.
                Please report your data and help us develop the library."
            );
        }

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
