use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use crate::{CMD_NAME, utils};

#[test]
fn no_color_display() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::testdata::grib2::jma_tornado_nowcast()?;
    let arg_path = tempfile.path();

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("dump").arg(arg_path).arg("0.0");
    let msg_colorized = cmd.output()?.stdout;
    let msg_colorized = String::from_utf8(msg_colorized)?;

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("dump").arg("--no-color").arg(arg_path).arg("0.0");
    cmd.assert()
        .success()
        .stdout(predicate::str::diff(msg_colorized))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn omitted_index_dumps_all_submessages() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::testdata::grib2::jma_tornado_nowcast()?;
    let mut expected = Vec::new();
    for index in 0..7 {
        let output = Command::cargo_bin(CMD_NAME)?
            .arg("dump")
            .arg("--no-color")
            .arg(tempfile.path())
            .arg(format!("0.{index}"))
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        expected.extend(output);
    }
    for options in [vec![], vec!["--no-color"]] {
        Command::cargo_bin(CMD_NAME)?
            .arg("dump")
            .args(options)
            .arg(tempfile.path())
            .assert()
            .success()
            .stdout(expected.clone())
            .stderr(predicate::str::is_empty());
    }
    Ok(())
}

#[test]
fn nonexistent_index_is_an_error() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::testdata::grib2::jma_tornado_nowcast()?;
    Command::cargo_bin(CMD_NAME)?
        .arg("dump")
        .arg(tempfile.path())
        .arg("0.7")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("no such index: 0.7"));
    Ok(())
}
