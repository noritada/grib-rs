use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

use crate::{utils, CMD_NAME};

macro_rules! test_subcommands_without_args {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr(
                    predicate::str::starts_with(
                        "error: the following required arguments were not provided:",
                    )
                        .and(predicate::str::contains("Usage:"))
                        .and(predicate::str::contains("Commands:").not()),
                );

            Ok(())
        }
    )*);
}

test_subcommands_without_args! {
    (decode_without_args, "decode"),
    (info_without_args, "info"),
    (list_without_args, "list"),
    (inspect_without_args, "inspect"),
}

macro_rules! test_subcommands_with_nonexisting_file {
    ($(($name:ident, $command:expr, $args:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let dir = TempDir::new()?;
            let file_path = dir.path().join("nosuchfile");
            let file_path = format!("{}", file_path.display());

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($command).arg(file_path).args($args);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty());

            Ok(())
        }
    )*);
}

test_subcommands_with_nonexisting_file! {
    (decode_with_nonexisting_file, "decode", vec!["1.1"]),
    (info_with_nonexisting_file, "info", Vec::<&str>::new()),
    (inspect_with_nonexisting_file, "inspect", Vec::<&str>::new()),
    (list_with_nonexisting_file, "list", Vec::<&str>::new()),
}

macro_rules! test_subcommands_with_wrong_input_files {
    ($(($name:ident, $command:expr, $input:expr, $args:expr, $stderr:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($command).arg(input.path()).args($args);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr($stderr);

            Ok(())
        }
    )*);
}

test_subcommands_with_wrong_input_files! {
    (
        decode_with_non_grib,
        "decode",
        utils::testdata::non_grib_file()?,
        vec!["1.1"],
        predicate::str::diff("error: Not GRIB data\n")
    ),
    (
        info_with_non_grib,
        "info",
        utils::testdata::non_grib_file()?,
        Vec::<&str>::new(),
        predicate::str::diff("error: Not GRIB data\n")
    ),
    (
        inspect_with_non_grib,
        "inspect",
        utils::testdata::non_grib_file()?,
        Vec::<&str>::new(),
        predicate::str::diff("error: Not GRIB data\n")
    ),
    (
        list_with_non_grib,
        "list",
        utils::testdata::non_grib_file()?,
        Vec::<&str>::new(),
        predicate::str::diff("error: Not GRIB data\n")
    ),
    (
        decode_with_empty_file,
        "decode",
        utils::testdata::empty_file()?,
        vec!["1.1"],
        predicate::str::diff("error: empty GRIB2 data\n")
    ),
    (
        info_with_empty_file,
        "info",
        utils::testdata::empty_file()?,
        Vec::<&str>::new(),
        predicate::str::diff("error: empty GRIB2 data\n")
    ),
    (
        inspect_with_empty_file,
        "inspect",
        utils::testdata::empty_file()?,
        Vec::<&str>::new(),
        predicate::str::diff("error: empty GRIB2 data\n")
    ),
    (
        list_with_empty_file,
        "list",
        utils::testdata::empty_file()?,
        Vec::<&str>::new(),
        predicate::str::diff("error: empty GRIB2 data\n")
    ),
    (
        decode_with_too_small_file,
        "decode",
        utils::testdata::too_small_file()?,
        vec!["1.1"],
        predicate::str::diff(
            "error: Read error: failed to fill whole buffer\n",
        )
    ),
    (
        info_with_too_small_file,
        "info",
        utils::testdata::too_small_file()?,
        Vec::<&str>::new(),
        predicate::str::diff(
            "error: Read error: failed to fill whole buffer\n",
        )
    ),
    (
        inspect_with_too_small_file,
        "inspect",
        utils::testdata::too_small_file()?,
        Vec::<&str>::new(),
        predicate::str::diff(
            "error: Read error: failed to fill whole buffer\n",
        )
    ),
    (
        list_with_too_small_file,
        "list",
        utils::testdata::too_small_file()?,
        Vec::<&str>::new(),
        predicate::str::diff(
            "error: Read error: failed to fill whole buffer\n",
        )
    ),
}
