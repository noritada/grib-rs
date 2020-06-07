use clap::{App, ArgMatches};

use crate::cli;

pub fn cli() -> Vec<App<'static, 'static>> {
    vec![decode::cli(), info::cli(), inspect::cli(), list::cli()]
}

pub fn dispatch(matches: ArgMatches<'static>) -> Result<(), cli::CliError> {
    match matches.subcommand() {
        ("decode", Some(args)) => decode::exec(args),
        ("info", Some(args)) => info::exec(args),
        ("inspect", Some(args)) => inspect::exec(args),
        ("list", Some(args)) => list::exec(args),
        ("", None) => unreachable!(),
        _ => unreachable!(),
    }
}

pub mod decode;
pub mod info;
pub mod inspect;
pub mod list;
