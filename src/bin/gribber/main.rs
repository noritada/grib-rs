use clap::{crate_name, crate_version, Command};

mod cli;
mod commands;

fn app() -> Command<'static> {
    Command::new(crate_name!())
        .version(crate_version!())
        .arg_required_else_help(true)
        .subcommands(commands::cli())
}

fn real_main() -> Result<(), cli::CliError> {
    let matches = app().get_matches();

    commands::dispatch(matches)
}

fn main() {
    if let Err(ref e) = real_main() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
