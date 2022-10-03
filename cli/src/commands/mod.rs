use clap::{ArgMatches, Command};

pub fn cli() -> Vec<Command> {
    vec![decode::cli(), info::cli(), inspect::cli(), list::cli()]
}

pub fn dispatch(matches: ArgMatches) -> anyhow::Result<()> {
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
