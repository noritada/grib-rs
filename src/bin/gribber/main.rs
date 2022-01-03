use clap::{crate_name, crate_version, App, AppSettings};

mod cli;
mod commands;

fn app() -> App<'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
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
