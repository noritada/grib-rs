use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

use crate::{CMD_NAME, utils};

macro_rules! test_operation_with_no_options {
    ($(($name:ident, $input:expr, $message_index:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("decode").arg(input.path()).arg($message_index);
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with("  Latitude   Longitude     Value\n"))
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
        utils::testdata::grib2::noaa_gdas_0_10()?,
        "2.0"
    ),
    (
        decoding_data_whose_grid_points_cannot_be_exported_as_latlons,
        utils::testdata::grib2::multi_message_data(3)?,
        "2.0"
    ),
}

macro_rules! test_operation_with_data_without_nan_values_and_byte_order_options {
    ($((
        $name:ident,
        $input:expr,
        $message_index:expr,
        $byte_order_flag:expr,
        $nan_replacement:expr,
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

            let actual = utils::get_uncompressed(&out_path)?;
            let expected: Vec<_> = $expected.chunks(4).flat_map($nan_replacement).collect();
            assert_eq!(actual, expected);

            Ok(())
        }
    )*);
}

test_operation_with_data_without_nan_values_and_byte_order_options! {
    (
        decoding_data_with_nan_values_as_big_endian,
        utils::testdata::grib2::jma_tornado_nowcast()?,
        "0.3",
        "-b",
        |b| match b {
            [0x62, 0x58, 0xd1, 0x9a] => vec![0x7f, 0xc0, 0x00, 0x00],
            b => b.to_vec(),
        },
        utils::testdata::flat_binary::jma_tornado_nowcast_be()?
    ),
    (
        decoding_data_with_nan_values_as_little_endian,
        utils::testdata::grib2::jma_tornado_nowcast()?,
        "0.3",
        "-l",
        |b| match b {
            [0x9a, 0xd1, 0x58, 0x62] => vec![0x00, 0x00, 0xc0, 0x7f],
            b => b.to_vec(),
        },
        utils::testdata::flat_binary::jma_tornado_nowcast_le()?
    ),
}

#[test]
fn test_input_from_stdin_and_output_to_stdout() -> Result<(), Box<dyn std::error::Error>> {
    let input = utils::testdata::grib2::jma_kousa()?;
    let out_path = "-";
    let expected = utils::testdata::flat_binary::jma_kousa_be()?;

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg("-")
        .arg("0.3")
        .arg("-b")
        .arg(&out_path);
    let mut cmd = assert_cmd::Command::from_std(cmd);
    cmd.write_stdin(utils::get_uncompressed(input)?)
        .assert()
        .success()
        .stdout(predicate::eq(expected))
        .stderr(predicate::str::is_empty());

    Ok(())
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
