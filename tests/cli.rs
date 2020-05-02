use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod utils;

const CMD_NAME: &'static str = "rsgrib";

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("USAGE:")
                .and(predicate::str::contains("FLAGS:"))
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
    let help_msg = format!("{}", String::from_utf8(help_msg)?);

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::similar(help_msg));

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

#[test]
fn info() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "\
Originating/generating centre:          34
Originating/generating sub-centre:      0
GRIB Master Tables Version Number:      code '5' is not implemented
GRIB Local Tables Version Number:       code '1' is not implemented
Significance of Reference Time:         Analysis
Reference time of data:                 2016-08-22 02:00:00Z
Production status of processed data:    Operational products
Type of processed data:                 Analysis and forecast products
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("info").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn list() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "\
[
    SubMessage {
        section2: None,
        section3: Some(
            1,
        ),
        section4: Some(
            2,
        ),
        section5: Some(
            3,
        ),
        section6: Some(
            4,
        ),
        section7: Some(
            5,
        ),
    },
    SubMessage {
        section2: None,
        section3: Some(
            1,
        ),
        section4: Some(
            6,
        ),
        section5: Some(
            7,
        ),
        section6: Some(
            8,
        ),
        section7: Some(
            9,
        ),
    },
    SubMessage {
        section2: None,
        section3: Some(
            1,
        ),
        section4: Some(
            10,
        ),
        section5: Some(
            11,
        ),
        section6: Some(
            12,
        ),
        section7: Some(
            13,
        ),
    },
    SubMessage {
        section2: None,
        section3: Some(
            1,
        ),
        section4: Some(
            14,
        ),
        section5: Some(
            15,
        ),
        section6: Some(
            16,
        ),
        section7: Some(
            17,
        ),
    },
    SubMessage {
        section2: None,
        section3: Some(
            1,
        ),
        section4: Some(
            18,
        ),
        section5: Some(
            19,
        ),
        section6: Some(
            20,
        ),
        section7: Some(
            21,
        ),
    },
    SubMessage {
        section2: None,
        section3: Some(
            1,
        ),
        section4: Some(
            22,
        ),
        section5: Some(
            23,
        ),
        section6: Some(
            24,
        ),
        section7: Some(
            25,
        ),
    },
    SubMessage {
        section2: None,
        section3: Some(
            1,
        ),
        section4: Some(
            26,
        ),
        section5: Some(
            27,
        ),
        section6: Some(
            28,
        ),
        section7: Some(
            29,
        ),
    },
]
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("list").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn templates() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "\
3.0
4.0
5.200
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("templates").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

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
    (templates_without_args, "templates"),
}

macro_rules! test_subcommands_with_nonexisting_file {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let dir = TempDir::new()?;
            let filename = format!("{}/nosuchfile", dir.path().display());

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str).arg(filename);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::starts_with(
                    "No such file or directory (os error 2):",
                ));

            Ok(())
        }
    )*);
}

test_subcommands_with_nonexisting_file! {
    (info_with_nonexisting_file, "info"),
    (list_with_nonexisting_file, "list"),
    (templates_with_nonexisting_file, "templates"),
}

macro_rules! test_subcommands_with_non_grib {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let tempfile = utils::non_grib_file()?;
            let arg_path = tempfile.path();

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str).arg(arg_path);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::similar("Not GRIB data\n"));

            Ok(())
        }
    )*);
}

test_subcommands_with_non_grib! {
    (info_with_non_grib, "info"),
    (list_with_non_grib, "list"),
    (templates_with_non_grib, "templates"),
}
