use crate::{Grib2SubmessageDecoder, test_utils};

macro_rules! test_operation_with_data_without_nan_values_and_byte_order_options {
    ($(($name:ident, $input:expr, $message_index:expr, $expected:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let buf = $input;

            let cursor = std::io::Cursor::new(buf);
            let grib2 = crate::from_reader(cursor)?;
            let (_index, submessage) = grib2
                .iter()
                .find(|(index, _submessage)| *index == $message_index)
                .ok_or_else(|| "message is not found")?;
            let decoder = Grib2SubmessageDecoder::from(submessage)?;
            let actual = decoder.dispatch()?.collect::<Vec<_>>();

            let expected = $expected
                .chunks(4)
                .map(|quad| f32::from_le_bytes(quad.try_into().unwrap())) // should be safely unwrapped
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_without_nan_values_and_byte_order_options! {
    (
        decoding_simple_packing_as_little_endian,
        test_utils::data::grib2::jma_kousa()?,
        (0, 3),
        test_utils::data::flat_binary::jma_kousa_le()?
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_1_as_little_endian,
        test_utils::data::grib2::noaa_gdas_12()?,
        (0, 0),
        test_utils::data::flat_binary::noaa_gdas_12_le()?
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_2_as_little_endian,
        test_utils::data::grib2::jma_meps()?,
        (0, 2),
        test_utils::data::flat_binary::jma_meps_le()?
    ),
    (
        decoding_complex_packing_where_nbit_is_zero,
        test_utils::data::grib2::noaa_gdas_46()?,
        (0, 0),
        test_utils::data::flat_binary::noaa_gdas_46_le()?
    ),
    (
        decoding_png_packing_with_num_bits_being_8_as_little_endian,
        test_utils::data::grib2::noaa_mrms_precip_flag()?,
        (0, 0),
        test_utils::data::flat_binary::noaa_mrms_precip_flag_le()?
    ),
    (
        decoding_png_packing_with_num_bits_being_16_as_little_endian,
        test_utils::data::grib2::noaa_mrms_reflectivity()?,
        (0, 0),
        test_utils::data::flat_binary::noaa_mrms_reflectivity_le()?
    ),
    (
        decoding_ccsds_compression_as_little_endian,
        test_utils::data::grib2::ecmwf_realtime_oper_fc_0()?,
        (0, 0),
        test_utils::data::flat_binary::ecmwf_realtime_oper_fc_0_le()?
    ),
    (
        decoding_ccsds_compression_where_num_bits_is_multiple_of_eight,
        test_utils::data::grib2::ecmwf_realtime_oper_fc_89()?,
        (0, 0),
        test_utils::data::flat_binary::ecmwf_realtime_oper_fc_89_le()?
    ),
}

macro_rules! test_operation_with_data_with_nan_values_as_little_endian {
    ($(($name:ident, $input:expr, $message_index:expr, $expected:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let buf = $input;

            let cursor = std::io::Cursor::new(buf);
            let grib2 = crate::from_reader(cursor)?;
            let (_index, submessage) = grib2
                .iter()
                .find(|(index, _submessage)| *index == $message_index)
                .ok_or_else(|| "message is not found")?;
            let decoder = Grib2SubmessageDecoder::from(submessage)?;
            let actual = decoder.dispatch()?.collect::<Vec<_>>();

            let expected = $expected
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

test_operation_with_data_with_nan_values_as_little_endian! {
    (
        decoding_run_length_packing_as_little_endian,
        test_utils::data::grib2::jma_tornado_nowcast()?,
        (0, 3),
        test_utils::data::flat_binary::jma_tornado_nowcast_le()?
    ),
    (
        decoding_simple_packing_with_bitmap_as_little_endian,
        test_utils::data::grib2::jma_msmguid()?,
        (0, 0),
        test_utils::data::flat_binary::jma_msmguid_le()?
    ),
    (
        decoding_complex_packing_with_first_order_spatial_differencing_as_little_endian,
        test_utils::data::grib2::ncmrwf_wind_solar()?,
        (0, 0),
        test_utils::data::flat_binary::ncmrwf_wind_solar_le()?
    ),
    (
        decoding_complex_packing_without_spatial_differencing_as_little_endian,
        test_utils::data::grib2::noaa_ndfd_critfireo()?,
        (0, 0),
        test_utils::data::flat_binary::noaa_ndfd_critfireo_0_le()?
    ),
    (
        decoding_complex_packing_without_spatial_differencing_as_little_endian_when_nbit_is_zero,
        test_utils::data::grib2::noaa_ndfd_critfireo()?,
        (1, 0),
        test_utils::data::flat_binary::noaa_ndfd_critfireo_1_le()?
    ),
    (
        decoding_complex_packing_with_missing_value_management_as_little_endian,
        test_utils::data::grib2::noaa_ndfd_minrh()?,
        (0, 0),
        test_utils::data::flat_binary::noaa_ndfd_minrh_0_le()?
    ),
}

// Compares integer values encoded using simple packing since there are some
// differences between float values from gribber and wgrib2.
macro_rules! test_operation_with_data_without_nan_values_compared_using_simple_packing {
    ($((
        $name:ident,
        $input:expr,
        $message_index:expr,
        $ref_val:expr,
        $exp:expr,
        $dig:expr,
        $expected:expr
    ),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let buf = $input;

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
            let expected = $expected;
            let expected = test_utils::encode_le_bytes_using_simple_packing(expected, ref_val, exp, dig);
            let actual = test_utils::encode_using_simple_packing(actual, ref_val, exp, dig);
            assert_eq!(actual, expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_without_nan_values_compared_using_simple_packing! {
    (
        decoding_jpeg2000_code_stream_as_little_endian,
        test_utils::data::grib2::cmc_glb()?,
        (0, 0),
        f32::from_be_bytes([0x45, 0x0e, 0xcc, 0x05]),
        -2,
        1,
        test_utils::data::flat_binary::cmc_glb_le()?
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_3_as_little_endian,
        test_utils::data::grib2::noaa_gdas_0_10()?,
        (1, 0),
        f32::from_be_bytes([0x00, 0x00, 0x00, 0x00]),
        1,
        8,
        test_utils::data::flat_binary::noaa_gdas_1_le()?
    ),
    (
        decoding_complex_packing_with_3_byte_spatial_differencing_extra_descriptors_starting_from_0x80,
        test_utils::data::grib2::noaa_gdas_0_10()?,
        (0, 0),
        f32::from_be_bytes([0x49, 0x67, 0xe7, 0xdf]),
        1,
        1,
        test_utils::data::flat_binary::noaa_gdas_0_le()?
    ),
    (
        decoding_complex_packing_with_zero_width_groups_as_little_endian,
        test_utils::data::grib2::noaa_gdas_0_10()?,
        (2, 0),
        f32::from_be_bytes([0x00, 0x00, 0x00, 0x00]),
        3,
        9,
        test_utils::data::flat_binary::noaa_gdas_2_le()?
    ),
    (
        decoding_png_packing_with_num_bits_being_24_as_little_endian,
        test_utils::data::grib2::noaa_mrms_merged_rho_hv()?,
        (0, 0),
        f32::from_be_bytes([0xc7, 0xc3, 0x1e, 0x00]),
        0,
        2,
        test_utils::data::flat_binary::noaa_mrms_merged_rho_hv_le()?
    ),
}
