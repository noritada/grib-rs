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
            cmd.assert().success().stderr(predicate::str::is_empty());

            Ok(())
        }
    )*);
}

test_operation_with_no_options! {
    (
        decoding_simple_packing,
        utils::jma_kousa_file()?,
        "0.3"
    ),
    (
        decoding_complex_packing,
        utils::jma_meps_file()?,
        "0.2"
    ),
    (
        decoding_run_length_encoding,
        utils::jma_tornado_nowcast_file()?,
        "0.3"
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
        utils::jma_kousa_file()?,
        "0.3",
        "-b",
        utils::kousa_be_bin_bytes()?
    ),
    (
        decoding_simple_packing_as_little_endian,
        utils::jma_kousa_file()?,
        "0.3",
        "-l",
        utils::kousa_le_bin_bytes()?
    ),
    (
        decoding_complex_packing_as_little_endian,
        utils::jma_meps_file()?,
        "0.2",
        "-l",
        utils::meps_le_bin_bytes()?
    ),
}

#[test]
fn decoding_run_length_packing_as_big_endian() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
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

    let expected = utils::tornado_nowcast_be_bin_bytes()?;
    let expected: Vec<_> = expected
        .chunks(4)
        .into_iter()
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
        utils::jma_tornado_nowcast_file()?,
        "0.3",
        "-l",
        utils::tornado_nowcast_le_bin_bytes()?
    ),
    (
        decoding_simple_packing_with_bitmap_as_little_endian,
        utils::jma_msmguid_file()?,
        "0.0",
        "-l",
        utils::msmguid_le_bin_bytes()?
    ),
}

#[test]
fn decoding_jpeg2001_code_stream_as_little_endian() -> Result<(), Box<dyn std::error::Error>> {
    let arg_path = utils::cmc_glb_file_path();

    let dir = TempDir::new()?;
    let out_path = dir.path().join("out.bin");
    let out_path = format!("{}", out_path.display());

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg(arg_path)
        .arg("0.0")
        .arg("-l")
        .arg(&out_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    // Compares integer values encoded using simple packing since there are some
    // differences between float values from gribber and wgrib2.
    let ref_val = f32::from_be_bytes([0x45, 0x0e, 0xcc, 0x05]);
    let exp: i16 = -2;
    let dig: i16 = 1;
    let expected = utils::cmc_glb_le_bin_bytes()?;
    let expected = utils::encode_le_bytes_using_simple_packing(expected, ref_val, exp, dig);
    let actual = utils::cat_as_bytes(&out_path)?;
    let actual = utils::encode_le_bytes_using_simple_packing(actual, ref_val, exp, dig);
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn trial_to_decode_submessage_with_nonexisting_index() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_kousa_file()?;
    let arg_path = tempfile.path();

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode").arg(arg_path).arg("0.9999");
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::diff("error: no such index: 0.9999\n"));

    Ok(())
}
