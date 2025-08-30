use crate::{
    decoder::{
        param::SimplePackingParam,
        simple::{SimplePackingDecodeIterator, SimplePackingDecodeIteratorWrapper},
        stream::{FixedValueIterator, NBitwiseIterator},
    },
    DecodeError, Grib2SubmessageDecoder, GribError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PngDecodeError {
    NotSupported,
    PngError(String),
}

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIteratorWrapper<impl Iterator<Item = u32> + '_>, GribError> {
    let sect5_data = &target.sect5_bytes;
    let param = SimplePackingParam::from_buf(&sect5_data[11..21])?;

    let buf = read_image_buffer(target.sect7_payload()).map_err(|e| {
        GribError::DecodeError(DecodeError::PngDecodeError(PngDecodeError::PngError(
            e.to_string(),
        )))
    })?;

    if param.nbit == 0 {
        eprintln!(
            "WARNING: nbit = 0 for PNG decoder is not tested.
            Please report your data and help us develop the library."
        );
        let decoder = SimplePackingDecodeIteratorWrapper::FixedValue(FixedValueIterator::new(
            param.zero_bit_reference_value(),
            target.num_points_encoded,
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
    let iter = SimplePackingDecodeIterator::new(iter, &param);
    let iter = SimplePackingDecodeIteratorWrapper::SimplePacking(iter);
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
