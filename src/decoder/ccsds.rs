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
            let element_size_in_bytes = super::helpers::num_octets(template.simple.num_bits);
            let size = element_size_in_bytes * target.num_encoded_points();
            let mut decoded = vec![0; size];
            let mut stream = backend::Stream::new(
                template.simple.num_bits.into(),
                template.block_size.into(),
                template.ref_sample_interval.into(),
                template.mask.into(),
            );
            stream.set_output_samples(target.num_encoded_points());
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
    #[cfg(all(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    use crate::def::grib2::{DataRepresentationTemplate, template::Template5_42};

    fn read_xz(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let f = File::open(path)?;
        let f = BufReader::new(f);
        let mut f = xz2::bufread::XzDecoder::new(f);
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }

    #[cfg(feature = "ccsds-unpack-with-rust-aec")]
    fn decoded_values_to_le_bytes(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
        values.into_iter().flat_map(f32::to_le_bytes).collect()
    }

    #[cfg(feature = "ccsds-unpack-with-rust-aec")]
    fn first_submessage_values(path: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let buf = read_xz(path)?;
        let cursor = std::io::Cursor::new(buf);
        let grib2 = crate::from_reader(cursor)?;
        let (_index, submessage) = grib2.iter().next().ok_or("GRIB file has no submessages")?;
        let decoder = Grib2SubmessageDecoder::from(submessage)?;
        Ok(decoder.dispatch()?.collect())
    }

    #[cfg(all(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    fn decoded_size(target: &Grib2SubmessageDecoder, template: &Template5_42) -> usize {
        let element_size_in_bytes = super::super::helpers::num_octets(template.simple.num_bits);
        element_size_in_bytes * target.num_encoded_points()
    }

    #[cfg(all(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    fn decode_with_libaec(
        target: &Grib2SubmessageDecoder,
        template: &Template5_42,
    ) -> Result<Vec<u8>, DecodeError> {
        let mut decoded = vec![0; decoded_size(target, template)];
        let mut stream = libaec::Stream::new(
            template.simple.num_bits.into(),
            template.block_size.into(),
            template.ref_sample_interval.into(),
            template.mask.into(),
        );
        stream.set_output_samples(target.num_encoded_points());
        stream
            .decode(target.sect7_payload(), &mut decoded)
            .map_err(DecodeError::from)?;
        Ok(decoded)
    }

    #[cfg(all(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    fn decode_with_rust_aec(
        target: &Grib2SubmessageDecoder,
        template: &Template5_42,
    ) -> Result<Vec<u8>, DecodeError> {
        let mut decoded = vec![0; decoded_size(target, template)];
        let mut stream = rust_aec::Stream::new(
            template.simple.num_bits.into(),
            template.block_size.into(),
            template.ref_sample_interval.into(),
            template.mask.into(),
        );
        stream.set_output_samples(target.num_encoded_points());
        stream
            .decode(target.sect7_payload(), &mut decoded)
            .map_err(DecodeError::from)?;
        Ok(decoded)
    }

    #[cfg(all(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    fn first_ccsds_decoder_with_nonzero_bits(
        path: &str,
    ) -> Result<Grib2SubmessageDecoder, Box<dyn std::error::Error>> {
        let buf = read_xz(path)?;
        let cursor = std::io::Cursor::new(buf);
        let grib2 = crate::from_reader(cursor)?;

        for (_index, submessage) in grib2.iter() {
            let decoder = Grib2SubmessageDecoder::from(submessage)?;
            if let DataRepresentationTemplate::_5_42(template) =
                &decoder.section5().payload.template
            {
                if template.simple.num_bits > 0 {
                    return Ok(decoder);
                }
            }
        }

        Err("non-zero-bit CCSDS submessage not found".into())
    }

    #[test]
    fn decode_ccsds_compression_when_nbit_is_zero() -> Result<(), Box<dyn std::error::Error>> {
        let buf = read_xz("testdata/20240101000000-0h-oper-fc.grib2.0-10.xz")?;

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

    #[test]
    #[cfg(feature = "ccsds-unpack-with-rust-aec")]
    fn decode_ecmwf_ccsds_0_matches_wgrib2_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let values = first_submessage_values("testdata/20240101000000-0h-oper-fc.grib2.0-10.xz")?;
        let actual = decoded_values_to_le_bytes(values);
        let expected = read_xz("testdata/gen/ecmwf-realtime-oper-fc-0-le.bin.xz")?;

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    #[cfg(feature = "ccsds-unpack-with-rust-aec")]
    fn decode_ecmwf_ccsds_89_matches_wgrib2_fixture() -> Result<(), Box<dyn std::error::Error>> {
        let values = first_submessage_values("testdata/20250912120000-0h-oper-fc.grib2.89.xz")?;
        let actual = decoded_values_to_le_bytes(values);
        let expected = read_xz("testdata/gen/ecmwf-realtime-oper-fc-89-le.bin.xz")?;

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    #[cfg(all(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    fn rust_aec_matches_libaec_for_ecmwf_ccsds_payload() -> Result<(), Box<dyn std::error::Error>> {
        let decoder = first_ccsds_decoder_with_nonzero_bits(
            "testdata/20250912120000-0h-oper-fc.grib2.89.xz",
        )?;
        let DataRepresentationTemplate::_5_42(template) = &decoder.section5().payload.template
        else {
            return Err("expected CCSDS template".into());
        };

        let libaec = decode_with_libaec(&decoder, template).map_err(|err| format!("{err:?}"))?;
        let rust_aec =
            decode_with_rust_aec(&decoder, template).map_err(|err| format!("{err:?}"))?;

        assert_eq!(rust_aec, libaec);
        Ok(())
    }

    #[test]
    #[cfg(all(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    fn rust_aec_backend_takes_priority_when_both_features_are_enabled() {
        let stream = backend::Stream::new(8, 8, 128, 0);

        fn assert_rust_aec_stream(_: &rust_aec::Stream) {}
        assert_rust_aec_stream(&stream);
    }
}

#[cfg_attr(feature = "ccsds-unpack-with-rust-aec", allow(dead_code))]
#[cfg(feature = "ccsds-unpack-with-libaec")]
mod libaec;
#[cfg(feature = "ccsds-unpack-with-rust-aec")]
mod rust_aec;

#[cfg(all(
    feature = "ccsds-unpack-with-libaec",
    not(feature = "ccsds-unpack-with-rust-aec")
))]
use libaec as backend;
#[cfg(feature = "ccsds-unpack-with-rust-aec")]
use rust_aec as backend;
