use crate::{
    decoder::{
        param::{CcsdsCompressionParam, SimplePackingParam},
        simple::*,
        stream::{FixedValueIterator, NBitwiseIterator},
        Grib2SubmessageDecoder,
    },
    error::*,
};

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIteratorWrapper<impl Iterator<Item = u32> + '_>, GribError> {
    let sect5_data = &target.sect5_payload;
    let simple_param = SimplePackingParam::from_buf(&sect5_data[6..16])?;
    let ccsds_param = CcsdsCompressionParam::from_buf(&sect5_data[16..20]);

    let decoder = if simple_param.nbit == 0 {
        SimplePackingDecodeIteratorWrapper::FixedValue(FixedValueIterator::new(
            simple_param.zero_bit_reference_value(),
            target.num_points_encoded,
        ))
    } else {
        let element_size_in_bytes = usize::from(simple_param.nbit >> 3) + 1;
        let size = element_size_in_bytes * target.num_points_encoded;
        let mut decoded = vec![0; size];
        let mut stream = libaec_sys::Stream::new(
            simple_param.nbit.into(),
            ccsds_param.block_size.into(),
            ccsds_param.reference_sample_interval.into(),
            ccsds_param.mask.into(),
        );
        stream
            .decode(&target.sect7_payload, &mut decoded)
            .map_err(|e| GribError::DecodeError(crate::DecodeError::Unknown(e.to_owned())))?;

        let decoder = NBitwiseIterator::new(decoded.into_iter(), element_size_in_bytes * 8);
        let decoder = SimplePackingDecodeIterator::new(decoder, &simple_param);
        let decoder = SimplePackingDecodeIteratorWrapper::SimplePacking(decoder);
        decoder
    };
    Ok(decoder)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Cursor, Read},
    };

    use super::*;
    use crate::context::from_reader;

    #[test]
    fn decode_ccsds_compression_when_nbit_is_zero() -> Result<(), Box<dyn std::error::Error>> {
        let f = File::open("testdata/20240101000000-0h-oper-fc.grib2.0-10.xz")?;
        let f = BufReader::new(f);
        let mut f = xz2::bufread::XzDecoder::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let f = Cursor::new(buf);

        let grib = from_reader(f)?;
        let message_index = (2, 0);
        let (_, submessage) = grib
            .iter()
            .find(|(index, _)| *index == message_index)
            .unwrap();
        let decoder = Grib2SubmessageDecoder::from(submessage)?;
        // Runs `decode()` internally.
        let actual = decoder.dispatch()?.collect::<Vec<_>>();
        let expected = vec![0f32; 0x0006318c];
        assert_eq!(actual, expected);

        Ok(())
    }
}
