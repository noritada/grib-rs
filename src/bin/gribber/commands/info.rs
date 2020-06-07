use clap::{App, Arg, ArgMatches, SubCommand};

use crate::cli;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("info")
        .about("Shows identification information")
        .arg(Arg::with_name("file").required(true))
}

pub fn exec(subcommand_matches: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = subcommand_matches.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;
    println!("{}", grib);
    Ok(())
}
