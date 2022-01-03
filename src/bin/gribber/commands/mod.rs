use clap::{App, ArgMatches};

use crate::cli;

pub fn cli() -> Vec<App<'static>> {
    vec![decode::cli(), info::cli(), inspect::cli(), list::cli()]
}

pub fn dispatch(matches: ArgMatches) -> Result<(), cli::CliError> {
    match matches.subcommand() {
        Some(("decode", args)) => decode::exec(args),
        Some(("info", args)) => info::exec(args),
        Some(("inspect", args)) => inspect::exec(args),
        Some(("list", args)) => list::exec(args),
        _ => unreachable!(),
    }
}

pub mod decode;
pub mod info;
pub mod inspect;
pub mod list;
