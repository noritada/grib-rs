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
