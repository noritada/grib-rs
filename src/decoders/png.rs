use crate::{
    decoders::{
        param::SimplePackingParam,
        simple::{SimplePackingDecodeError, SimplePackingDecodeIterator},
        stream::NBitwiseIterator,
    },
    utils::read_as,
    DecodeError, Grib2SubmessageDecoder, GribError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PngDecodeError {
    NotSupported,
    PngError(String),
}

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIterator<impl Iterator<Item = u32> + '_>, GribError> {
    let sect5_data = &target.sect5_payload;
    let param = SimplePackingParam::from_buf(&sect5_data[6..15]);
    let value_type = read_as!(u8, sect5_data, 15);

    if value_type != 0 {
        return Err(GribError::DecodeError(
            DecodeError::SimplePackingDecodeError(
                SimplePackingDecodeError::OriginalFieldValueTypeNotSupported,
            ),
        ));
    }

    let buf = read_image_buffer(&target.sect7_payload).map_err(|e| {
        GribError::DecodeError(DecodeError::PngDecodeError(PngDecodeError::PngError(
            e.to_string(),
        )))
    })?;

    if param.nbit != 16 {
        eprintln!(
            "WARNING: nbit != 16 for PNG decoder is not tested.
            Please report your data and help us develop the library."
        );
    }

    let iter = NBitwiseIterator::new(buf, usize::from(param.nbit));
    let iter = SimplePackingDecodeIterator::new(iter, &param);
    Ok(iter)
}

fn read_image_buffer(buf: &[u8]) -> Result<Vec<u8>, png::DecodingError> {
    let reader = std::io::Cursor::new(&buf);
    let decoder = png::Decoder::new(reader);
    let mut reader = decoder.read_info()?;
    let mut out_buf = vec![0; reader.output_buffer_size()];
    let _info = reader.next_frame(&mut out_buf)?;
    Ok(out_buf)
}
