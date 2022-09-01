use crate::{utils, CMD_NAME};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn info() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::testdata::grib2::jma_tornado_nowcast()?;
    let arg_path = tempfile.path();

    let out_str = "\
Discipline:                             Meteorological products
Total Length:                           10321
Originating/generating centre:          Tokyo (RSMC), Japan Meteorological Agency
Originating/generating sub-centre:      0
GRIB Master Tables Version Number:      5 (4 November 2009)
GRIB Local Tables Version Number:       1 (Number of local tables version used)
Significance of Reference Time:         Analysis
Reference time of data:                 2016-08-22 02:00:00 UTC
Production status of processed data:    Operational products
Type of processed data:                 Analysis and forecast products
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("info").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::diff(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}
