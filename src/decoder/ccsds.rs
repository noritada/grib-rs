use crate::{
    DecodeError, Grib2GpvUnpack,
    decoder::{
        Grib2SubmessageDecoder,
        simple::*,
        stream::{FixedValueIterator, NBitwiseIterator},
    },
};

pub(crate) struct Ccsds<'d>(
    pub(crate) &'d Grib2SubmessageDecoder,
    pub(crate) &'d crate::def::grib2::template::Template5_42,
);

impl<'d> Grib2GpvUnpack for Ccsds<'d> {
    type Iter<'a>
        = SimplePackingDecoder<NBitwiseIterator<Vec<u8>>>
    where
        Self: 'a;

    fn iter<'a>(&'a self) -> Result<Self::Iter<'a>, DecodeError> {
        let Self(target, template) = self;
        super::orig_field_type_is_supported(template.orig_field_type)?;

        let decoder = if template.simple.num_bits == 0 {
            SimplePackingDecoder::ZeroLength(FixedValueIterator::new(
                template.simple.zero_bit_reference_value(),
                target.num_encoded_points(),
            ))
        } else {
            let element_size_in_bytes = usize::from(template.simple.num_bits + 0b111) >> 3;
            let size = element_size_in_bytes * target.num_encoded_points();
            let mut decoded = vec![0; size];
            let mut stream = aec::Stream::new(
                template.simple.num_bits.into(),
                template.block_size.into(),
                template.ref_sample_interval.into(),
                template.mask.into(),
            );
            stream
                .decode(target.sect7_payload(), &mut decoded)
                .map_err(DecodeError::from)?;

            let decoder = NBitwiseIterator::new(decoded, element_size_in_bytes * 8);
            let decoder = NonZeroSimplePackingDecoder::new(decoder, &template.simple);
            SimplePackingDecoder::NonZeroLength(decoder)
        };
        Ok(decoder)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::*;

    #[test]
    fn decode_ccsds_compression_when_nbit_is_zero() -> Result<(), Box<dyn std::error::Error>> {
        let f = File::open("testdata/20240101000000-0h-oper-fc.grib2.0-10.xz")?;
        let f = BufReader::new(f);
        let mut f = xz2::bufread::XzDecoder::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        // submessage 2.0
        let decoder = Grib2SubmessageDecoder::new(
            405900,
            buf[0x0006870b..0x00068724].to_vec(),
            buf[0x00068724..0x0006872a].to_vec(),
            buf[0x0006872a..0x0006872f].to_vec(),
        )?;
        // Runs `decode()` internally.
        let actual = decoder.dispatch()?.collect::<Vec<_>>();
        let expected = vec![0f32; 0x0006318c];
        assert_eq!(actual, expected);

        Ok(())
    }
}

mod aec;
