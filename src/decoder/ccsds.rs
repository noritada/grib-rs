use crate::{
    decoder::{
        param::SimplePackingParam, simple::*, stream::FixedValueIterator, Grib2SubmessageDecoder,
    },
    error::*,
};

pub(crate) fn decode(
    target: &Grib2SubmessageDecoder,
) -> Result<SimplePackingDecodeIteratorWrapper<impl Iterator<Item = u32> + '_>, GribError> {
    let sect5_data = &target.sect5_payload;
    let simple_param = SimplePackingParam::from_buf(&sect5_data[6..16])?;

    let decoder = if simple_param.nbit == 0 {
        SimplePackingDecodeIteratorWrapper::<std::ops::Range<u32>>::FixedValue(
            FixedValueIterator::new(
                simple_param.zero_bit_reference_value(),
                target.num_points_encoded,
            ),
        ) // FIXME: `std::ops::Range<u32>` is dummy because the code is partly
          // unimplemented
    } else {
        unimplemented!()
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
