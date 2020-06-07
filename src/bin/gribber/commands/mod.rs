use clap::{App, ArgMatches};

use crate::cli;

pub fn cli() -> Vec<App<'static, 'static>> {
    vec![info::cli(), list::cli(), inspect::cli(), decode::cli()]
}

pub fn dispatch(matches: ArgMatches<'static>) -> Result<(), cli::CliError> {
    match matches.subcommand() {
        ("info", Some(subcommand_matches)) => info::exec(subcommand_matches),
        ("list", Some(subcommand_matches)) => list::exec(subcommand_matches),
        ("inspect", Some(subcommand_matches)) => inspect::exec(subcommand_matches),
        ("decode", Some(subcommand_matches)) => decode::exec(subcommand_matches),
        ("", None) => unreachable!(),
        _ => unreachable!(),
    }
}

pub mod decode;
pub mod info;
pub mod inspect;
pub mod list;
