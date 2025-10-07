use clap::{ArgMatches, Command};

pub fn cli() -> Vec<Command> {
    vec![
        completions::cli(),
        decode::cli(),
        dump::cli(),
        info::cli(),
        inspect::cli(),
        list::cli(),
    ]
}

pub fn dispatch(matches: ArgMatches) -> anyhow::Result<()> {
    match matches.subcommand() {
        Some(("completions", args)) => completions::exec(args),
        Some(("decode", args)) => decode::exec(args),
        Some(("dump", args)) => dump::exec(args),
        Some(("info", args)) => info::exec(args),
        Some(("inspect", args)) => inspect::exec(args),
        Some(("list", args)) => list::exec(args),
        _ => unreachable!(),
    }
}

pub mod completions;
pub mod decode;
pub mod dump;
pub mod info;
pub mod inspect;
pub mod list;
