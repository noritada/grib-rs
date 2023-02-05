use crate::{utils, CMD_NAME};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

macro_rules! test_operation_with_no_options {
    ($(($name:ident, $input:expr, $message_index:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("decode").arg(input.path()).arg($message_index);
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(" Latitude Longitude     Value\n"))
                .stderr(predicate::str::is_empty());

            Ok(())
        }
    )*);
}

test_operation_with_no_options! {
    (
        decoding_simple_packing,
        utils::testdata::grib2::jma_kousa()?,
        "0.3"
    ),
    (
        decoding_complex_packing,
        utils::testdata::grib2::jma_meps()?,
        "0.2"
    ),
    (
        decoding_run_length_encoding,
        utils::testdata::grib2::jma_tornado_nowcast()?,
        "0.3"
    ),
    (
        decoding_multi_message_data,
        utils::testdata::grib2::multi_message_data(3)?,
        "2.0"
    ),
}

macro_rules! test_operation_with_data_without_nan_values_and_byte_order_options {
    ($(($name:ident, $input:expr, $message_index:expr, $byte_order_flag:expr, $expected:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let dir = TempDir::new()?;
            let out_path = dir.path().join("out.bin");
            let out_path = format!("{}", out_path.display());

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("decode")
                .arg(input.path())
                .arg($message_index)
                .arg($byte_order_flag)
                .arg(&out_path);
            cmd.assert()
                .success()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::is_empty());

            let actual = utils::cat_as_bytes(&out_path)?;
            assert_eq!(actual, $expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_without_nan_values_and_byte_order_options! {
    (
        decoding_simple_packing_as_big_endian,
        utils::testdata::grib2::jma_kousa()?,
        "0.3",
        "-b",
        utils::testdata::flat_binary::jma_kousa_be()?
    ),
    (
        decoding_simple_packing_as_little_endian,
        utils::testdata::grib2::jma_kousa()?,
        "0.3",
        "-l",
        utils::testdata::flat_binary::jma_kousa_le()?
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_1_as_little_endian,
        utils::testdata::grib2::noaa_gdas_12()?,
        "0.0",
        "-l",
        utils::testdata::flat_binary::noaa_gdas_12_le()?
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_2_as_little_endian,
        utils::testdata::grib2::jma_meps()?,
        "0.2",
        "-l",
        utils::testdata::flat_binary::jma_meps_le()?
    ),
    (
        decoding_complex_packing_where_nbit_is_zero,
        utils::testdata::grib2::noaa_gdas_46()?,
        "0.0",
        "-l",
        utils::testdata::flat_binary::noaa_gdas_46_le()?
    ),
}

#[test]
fn decoding_run_length_packing_as_big_endian() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::testdata::grib2::jma_tornado_nowcast()?;
    let arg_path = tempfile.path();

    let dir = TempDir::new()?;
    let out_path = dir.path().join("out.bin");
    let out_path = format!("{}", out_path.display());

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg(arg_path)
        .arg("0.3")
        .arg("-b")
        .arg(&out_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = utils::testdata::flat_binary::jma_tornado_nowcast_be()?;
    let expected: Vec<_> = expected
        .chunks(4)
        .flat_map(|b| match b {
            [0x62, 0x58, 0xd1, 0x9a] => vec![0x7f, 0xc0, 0x00, 0x00],
            b => b.to_vec(),
        })
        .collect();
    let actual = utils::cat_as_bytes(&out_path)?;
    assert_eq!(actual, expected);

    Ok(())
}

macro_rules! test_operation_with_data_with_nan_values_as_little_endian {
    ($(($name:ident, $input:expr, $message_index:expr, $byte_order_flag:expr, $expected:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let dir = TempDir::new()?;
            let out_path = dir.path().join("out.bin");
            let out_path = format!("{}", out_path.display());

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("decode")
                .arg(input.path())
                .arg($message_index)
                .arg($byte_order_flag)
                .arg(&out_path);
            cmd.assert()
                .success()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::is_empty());

            let expected: Vec<_> = $expected
                .chunks(4)
                .into_iter()
                .flat_map(|b| match b {
                    [0x9a, 0xd1, 0x58, 0x62] => vec![0x00, 0x00, 0xc0, 0x7f],
                    b => b.to_vec(),
                })
                .collect();
            let actual = utils::cat_as_bytes(&out_path)?;
            assert_eq!(actual, expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_with_nan_values_as_little_endian! {
    (
        decoding_run_length_packing_as_little_endian,
        utils::testdata::grib2::jma_tornado_nowcast()?,
        "0.3",
        "-l",
        utils::testdata::flat_binary::jma_tornado_nowcast_le()?
    ),
    (
        decoding_simple_packing_with_bitmap_as_little_endian,
        utils::testdata::grib2::jma_msmguid()?,
        "0.0",
        "-l",
        utils::testdata::flat_binary::jma_msmguid_le()?
    ),
}

// Compares integer values encoded using simple packing since there are some
// differences between float values from gribber and wgrib2.
macro_rules! test_operation_with_data_without_nan_values_compared_using_simple_packing {
    ($((
        $name:ident,
        $input:expr,
        $message_index:expr,
        $byte_order_flag:expr,
        $ref_val:expr,
        $exp:expr,
        $dig:expr,
        $expected:expr
    ),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let dir = TempDir::new()?;
            let out_path = dir.path().join("out.bin");
            let out_path = format!("{}", out_path.display());

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("decode")
                .arg(input.path())
                .arg($message_index)
                .arg($byte_order_flag)
                .arg(&out_path);
            cmd.assert()
                .success()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::is_empty());

            let ref_val = $ref_val;
            let exp: i16 = $exp;
            let dig: i16 = $dig;
            let expected = $expected;
            let expected = utils::encode_le_bytes_using_simple_packing(expected, ref_val, exp, dig);
            let actual = utils::cat_as_bytes(&out_path)?;
            let actual = utils::encode_le_bytes_using_simple_packing(actual, ref_val, exp, dig);
            assert_eq!(actual, expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_without_nan_values_compared_using_simple_packing! {
    (
        decoding_jpeg2001_code_stream_as_little_endian,
        utils::testdata::grib2::cmc_glb()?,
        "0.0",
        "-l",
        f32::from_be_bytes([0x45, 0x0e, 0xcc, 0x05]),
        -2,
        1,
        utils::testdata::flat_binary::cmc_glb_le()?
    ),
    (
        decoding_complex_packing_with_num_descriptor_octet_being_3_as_little_endian,
        utils::testdata::grib2::noaa_gdas_0_10()?,
        "1.0",
        "-l",
        f32::from_be_bytes([0x00, 0x00, 0x00, 0x00]),
        1,
        8,
        utils::testdata::flat_binary::noaa_gdas_1_le()?
    ),
    (
        decoding_complex_packing_with_3_byte_spatial_differencing_extra_descriptors_starting_from_0x80,
        utils::testdata::grib2::noaa_gdas_0_10()?,
        "0.0",
        "-l",
        f32::from_be_bytes([0x49, 0x67, 0xe7, 0xdf]),
        1,
        1,
        utils::testdata::flat_binary::noaa_gdas_0_le()?
    ),
    (
        decoding_complex_packing_with_zero_width_groups_as_little_endian,
        utils::testdata::grib2::noaa_gdas_0_10()?,
        "2.0",
        "-l",
        f32::from_be_bytes([0x00, 0x00, 0x00, 0x00]),
        3,
        9,
        utils::testdata::flat_binary::noaa_gdas_2_le()?
    ),
}

macro_rules! test_trial_to_decode_nonexisting_submessage {
    ($(($name:ident, $input:expr, $message_index:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("decode").arg(input.path()).arg($message_index);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::contains("error: no such index:"));

            Ok(())
        }
    )*);
}

test_trial_to_decode_nonexisting_submessage! {
    (
        trial_to_decode_submessage_with_nonexisting_submessage_index,
        utils::testdata::grib2::jma_kousa()?,
        "0.999"
    ),
    (
        trial_to_decode_submessage_with_nonexisting_message_index,
        utils::testdata::grib2::jma_kousa()?,
        "1.0"
    ),
    (
        trial_to_decode_submessage_with_nonexisting_submessage_index_for_multi_message_data,
        utils::testdata::grib2::multi_message_data(3)?,
        "0.1"
    ),
    (
        trial_to_decode_submessage_with_nonexisting_message_index_for_multi_message_data,
        utils::testdata::grib2::multi_message_data(3)?,
        "999.0"
    ),
}
