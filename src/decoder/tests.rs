use crate::{Grib2SubmessageDecoder, test_utils};

macro_rules! test_operation_with_data_without_nan_values {
    (
        $(
            $(#[$meta:meta])*
            ($name:ident, $input:expr, $message_index:expr, $expected:expr),
        )*
    ) => ($(
        $(#[$meta])*
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let buf = crate::test_utils::decompress_to_vec($input)?;

            let cursor = std::io::Cursor::new(buf);
            let grib2 = crate::from_reader(cursor)?;
            let (_index, submessage) = grib2
                .iter()
                .find(|(index, _submessage)| *index == $message_index)
                .ok_or_else(|| "message is not found")?;
            let decoder = Grib2SubmessageDecoder::from(submessage)?;
            let actual = decoder.dispatch()?.collect::<Vec<_>>();

            let expected = crate::test_utils::decompress_to_vec($expected)?
                .chunks(4)
                .map(|quad| f32::from_le_bytes(quad.try_into().unwrap())) // should be safely unwrapped
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_without_nan_values! {
    (
        decoding_simple_packing,
        test_utils::data::grib2::JMA_KOUSA,
        (0, 3),
        test_utils::data::flat_binary::JMA_KOUSA_LE
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_1,
        test_utils::data::grib2::NOAA_GDAS_12,
        (0, 0),
        test_utils::data::flat_binary::NOAA_GDAS_12_LE
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_2,
        test_utils::data::grib2::JMA_MEPS,
        (0, 2),
        test_utils::data::flat_binary::JMA_MEPS_LE
    ),
    (
        decoding_complex_packing_where_nbit_is_zero,
        test_utils::data::grib2::NOAA_GDAS_46,
        (0, 0),
        test_utils::data::flat_binary::NOAA_GDAS_46_LE
    ),
    #[cfg(feature = "png-unpack-with-png-crate")]
    (
        decoding_png_packing_with_num_bits_being_8,
        test_utils::data::grib2::NOAA_MRMS_PRECIP_FLAG,
        (0, 0),
        test_utils::data::flat_binary::NOAA_MRMS_PRECIP_FLAG_LE
    ),
    #[cfg(feature = "png-unpack-with-png-crate")]
    (
        decoding_png_packing_with_num_bits_being_16,
        test_utils::data::grib2::NOAA_MRMS_REFLECTIVITY,
        (0, 0),
        test_utils::data::flat_binary::NOAA_MRMS_REFLECTIVITY_LE
    ),
    #[cfg(any(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    (
        decoding_ccsds_compression,
        test_utils::data::grib2::ECMWF_REALTIME_OPER_FC_0,
        (0, 0),
        test_utils::data::flat_binary::ECMWF_REALTIME_OPER_FC_0_LE
    ),
    #[cfg(any(
        feature = "ccsds-unpack-with-libaec",
        feature = "ccsds-unpack-with-rust-aec"
    ))]
    (
        decoding_ccsds_compression_where_num_bits_is_multiple_of_eight,
        test_utils::data::grib2::ECMWF_REALTIME_OPER_FC_89,
        (0, 0),
        test_utils::data::flat_binary::ECMWF_REALTIME_OPER_FC_89_LE
    ),
}

macro_rules! test_operation_with_data_with_nan_values {
    (
        $(
            $(#[$meta:meta])*
            ($name:ident, $input:expr, $message_index:expr, $expected:expr),
        )*
    ) => ($(
        $(#[$meta])*
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let buf = crate::test_utils::decompress_to_vec($input)?;

            let cursor = std::io::Cursor::new(buf);
            let grib2 = crate::from_reader(cursor)?;
            let (_index, submessage) = grib2
                .iter()
                .find(|(index, _submessage)| *index == $message_index)
                .ok_or_else(|| "message is not found")?;
            let decoder = Grib2SubmessageDecoder::from(submessage)?;
            let actual = decoder.dispatch()?.collect::<Vec<_>>();

            let expected = crate::test_utils::decompress_to_vec($expected)?
                .chunks(4)
                .map(|b| match b {
                    [0x9a, 0xd1, 0x58, 0x62] => [0x00, 0x00, 0xc0, 0x7f],
                    b => b.try_into().unwrap(),
                })
                .map(|quad| f32::from_le_bytes(quad.try_into().unwrap())) // should be safely unwrapped
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), expected.len());
            let all_matched = actual
                .iter()
                .zip(expected.iter())
                .all(|(a, e)| (a.is_nan() && e.is_nan()) || a == e);
            assert!(all_matched, "vectors differ");

            Ok(())
        }
    )*);
}

test_operation_with_data_with_nan_values! {
    (
        decoding_run_length_packing,
        test_utils::data::grib2::JMA_TORNADO_NOWCAST,
        (0, 3),
        test_utils::data::flat_binary::JMA_TORNADO_NOWCAST_LE
    ),
    (
        decoding_simple_packing_with_bitmap,
        test_utils::data::grib2::JMA_MSMGUID,
        (0, 0),
        test_utils::data::flat_binary::JMA_MSMGUID_LE
    ),
    (
        decoding_complex_packing_with_first_order_spatial_differencing,
        test_utils::data::grib2::NCMRWF_WIND_SOLAR,
        (0, 0),
        test_utils::data::flat_binary::NCMRWF_WIND_SOLAR_LE
    ),
    (
        decoding_complex_packing_without_spatial_differencing,
        test_utils::data::grib2::NOAA_NDFD_CRITFIREO,
        (0, 0),
        test_utils::data::flat_binary::NOAA_NDFD_CRITFIREO_0_LE
    ),
    (
        decoding_complex_packing_without_spatial_differencing_when_nbit_is_zero,
        test_utils::data::grib2::NOAA_NDFD_CRITFIREO,
        (1, 0),
        test_utils::data::flat_binary::NOAA_NDFD_CRITFIREO_1_LE
    ),
    (
        decoding_complex_packing_with_missing_value_management,
        test_utils::data::grib2::NOAA_NDFD_MINRH,
        (0, 0),
        test_utils::data::flat_binary::NOAA_NDFD_MINRH_0_LE
    ),
}

// Compares integer values encoded using simple packing since there are some
// differences between float values from gribber and wgrib2.
macro_rules! test_operation_with_data_without_nan_values_compared_using_simple_packing {
    (
        $(
            $(#[$meta:meta])*
            (
                $name:ident,
                $input:expr,
                $message_index:expr,
                $ref_val:expr,
                $exp:expr,
                $dig:expr,
                $expected:expr
            ),
        )*
    ) => ($(
        $(#[$meta])*
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let buf = crate::test_utils::decompress_to_vec($input)?;

            let cursor = std::io::Cursor::new(buf);
            let grib2 = crate::from_reader(cursor)?;
            let (_index, submessage) = grib2
                .iter()
                .find(|(index, _submessage)| *index == $message_index)
                .ok_or_else(|| "message is not found")?;
            let decoder = Grib2SubmessageDecoder::from(submessage)?;
            let actual = decoder.dispatch()?.collect::<Vec<_>>();

            let ref_val = $ref_val;
            let exp: i16 = $exp;
            let dig: i16 = $dig;
            let expected = crate::test_utils::decompress_to_vec($expected)?;
            let expected = test_utils::encode_le_bytes_using_simple_packing(expected, ref_val, exp, dig);
            let actual = test_utils::encode_using_simple_packing(actual, ref_val, exp, dig);
            assert_eq!(actual, expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_without_nan_values_compared_using_simple_packing! {
    #[cfg(any(
        feature = "jpeg2000-unpack-with-hayro",
        feature = "jpeg2000-unpack-with-openjpeg"
    ))]
    (
        decoding_jpeg2000_code_stream,
        test_utils::data::grib2::CMC_GLB,
        (0, 0),
        f32::from_be_bytes([0x45, 0x0e, 0xcc, 0x05]),
        -2,
        1,
        test_utils::data::flat_binary::CMC_GLB_LE
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_3,
        test_utils::data::grib2::NOAA_GDAS_0_10,
        (1, 0),
        f32::from_be_bytes([0x00, 0x00, 0x00, 0x00]),
        1,
        8,
        test_utils::data::flat_binary::NOAA_GDAS_1_LE
    ),
    (
        decoding_complex_packing_with_3_byte_spatial_differencing_extra_descriptors_starting_from_0x80,
        test_utils::data::grib2::NOAA_GDAS_0_10,
        (0, 0),
        f32::from_be_bytes([0x49, 0x67, 0xe7, 0xdf]),
        1,
        1,
        test_utils::data::flat_binary::NOAA_GDAS_0_LE
    ),
    (
        decoding_complex_packing_with_zero_width_groups,
        test_utils::data::grib2::NOAA_GDAS_0_10,
        (2, 0),
        f32::from_be_bytes([0x00, 0x00, 0x00, 0x00]),
        3,
        9,
        test_utils::data::flat_binary::NOAA_GDAS_2_LE
    ),
    #[cfg(feature = "png-unpack-with-png-crate")]
    (
        decoding_png_packing_with_num_bits_being_24,
        test_utils::data::grib2::NOAA_MRMS_MERGED_RHO_HV,
        (0, 0),
        f32::from_be_bytes([0xc7, 0xc3, 0x1e, 0x00]),
        0,
        2,
        test_utils::data::flat_binary::NOAA_MRMS_MERGED_RHO_HV_LE
    ),
}
