use crate::{
    DecodeError, Grib2GpvUnpack, Grib2SubmessageDecoder,
    decoder::{
        param::SimplePackingParam,
        simple::{NonZeroSimplePackingDecoder, SimplePackingDecoder},
        stream::{FixedValueIterator, NBitwiseIterator},
    },
};

pub(crate) struct Png<'d>(pub(crate) &'d Grib2SubmessageDecoder);

impl<'d> Grib2GpvUnpack for Png<'d> {
    type Iter<'a>
        = SimplePackingDecoder<NBitwiseIterator<Vec<u8>>>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError> {
        let Self(target) = self;
        let sect5_data = &target.sect5_bytes;
        let param = SimplePackingParam::from_buf(&sect5_data[11..21])?;

        let buf = read_image_buffer(target.sect7_payload())
            .map_err(|e| DecodeError::from(format!("PNG decode error: {e}")))?;

        if param.nbit == 0 {
            eprintln!(
                "WARNING: nbit = 0 for PNG decoder is not tested.
                Please report your data and help us develop the library."
            );
            let decoder = SimplePackingDecoder::ZeroLength(FixedValueIterator::new(
                param.zero_bit_reference_value(),
                target.num_points_encoded(),
            ));
            return Ok(decoder);
        };

        if param.nbit != 16 {
            eprintln!(
                "WARNING: nbit != 16 for PNG decoder is not tested.
                Please report your data and help us develop the library."
            );
        }

        let iter = NBitwiseIterator::new(buf, usize::from(param.nbit));
        let iter = NonZeroSimplePackingDecoder::new(iter, &param);
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
