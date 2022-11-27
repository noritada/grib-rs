use clap::Command;

mod cli;
mod commands;

pub(crate) fn app() -> Command {
    Command::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(true)
        .subcommands(commands::cli())
}

fn real_main() -> anyhow::Result<()> {
    let matches = app().get_matches();

    commands::dispatch(matches)
}

fn main() {
    if let Err(ref e) = real_main() {
        let red = console::Style::new().red();
        eprintln!("{}: {e}", red.apply_to("error"));
        std::process::exit(1);
    }
}
