use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod commands;
mod utils;

pub(crate) const CMD_NAME: &str = "gribber";

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("USAGE:")
                .and(predicate::str::contains("OPTIONS:"))
                .and(predicate::str::contains("SUBCOMMANDS:")),
        )
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn no_subcommand_specified() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("--help");
    let help_msg = cmd.output()?.stdout;
    let help_msg = String::from_utf8(help_msg)?;

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::diff(help_msg));

    Ok(())
}

#[test]
fn no_such_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("foo");
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::starts_with(
                "error: Found argument 'foo' which wasn't expected, or isn't valid in this context",
            )
            .and(predicate::str::contains("USAGE:"))
            .and(predicate::str::contains("SUBCOMMANDS:").not()),
        );

    Ok(())
}

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
                        "error: The following required arguments were not provided:",
                    )
                        .and(predicate::str::contains("USAGE:"))
                        .and(predicate::str::contains("SUBCOMMANDS:").not()),
                );

            Ok(())
        }
    )*);
}

test_subcommands_without_args! {
    (info_without_args, "info"),
    (list_without_args, "list"),
    (inspect_without_args, "inspect"),
}

macro_rules! test_subcommands_with_nonexisting_file {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let dir = TempDir::new()?;
            let file_path = dir.path().join("nosuchfile");
            let file_path = format!("{}", file_path.display());

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str).arg(file_path);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty());

            Ok(())
        }
    )*);
}

test_subcommands_with_nonexisting_file! {
    (info_with_nonexisting_file, "info"),
    (list_with_nonexisting_file, "list"),
    (inspect_with_nonexisting_file, "inspect"),
}

macro_rules! test_subcommands_with_wrong_input_files {
    ($(($name:ident, $command:expr, $input:expr, $stderr:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($command).arg(input.path());
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
        info_with_non_grib,
        "info",
        utils::testdata::non_grib_file()?,
        predicate::str::diff("error: Not GRIB data\n")
    ),
    (
        inspect_with_non_grib,
        "inspect",
        utils::testdata::non_grib_file()?,
        predicate::str::diff("error: Not GRIB data\n")
    ),
    (
        list_with_non_grib,
        "list",
        utils::testdata::non_grib_file()?,
        predicate::str::diff("error: Not GRIB data\n")
    ),
    (
        info_with_empty_file,
        "info",
        utils::testdata::empty_file()?,
        predicate::str::diff("error: empty GRIB2 data\n")
    ),
    (
        inspect_with_empty_file,
        "inspect",
        utils::testdata::empty_file()?,
        predicate::str::diff("error: empty GRIB2 data\n")
    ),
    (
        list_with_empty_file,
        "list",
        utils::testdata::empty_file()?,
        predicate::str::diff("error: empty GRIB2 data\n")
    ),
    (
        info_with_too_small_file,
        "info",
        utils::testdata::too_small_file()?,
        predicate::str::diff(
            "error: Read error: failed to fill whole buffer\n",
        )
    ),
    (
        inspect_with_too_small_file,
        "inspect",
        utils::testdata::too_small_file()?,
        predicate::str::diff(
            "error: Read error: failed to fill whole buffer\n",
        )
    ),
    (
        list_with_too_small_file,
        "list",
        utils::testdata::too_small_file()?,
        predicate::str::diff(
            "error: Read error: failed to fill whole buffer\n",
        )
    ),
}
