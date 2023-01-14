use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

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
            predicate::str::contains("Usage:")
                .and(predicate::str::contains("Options:"))
                .and(predicate::str::contains("Commands:")),
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
            predicate::str::starts_with("error: unrecognized subcommand 'foo'")
                .and(predicate::str::contains("Usage:"))
                .and(predicate::str::contains("Commands:").not()),
        );

    Ok(())
}
