use clap::{App, Arg, ArgMatches, SubCommand};

use crate::cli;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("list")
        .about("Lists contained data")
        .arg(Arg::with_name("file").required(true))
}

pub fn exec(args: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = args.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;
    for (i, submessage) in grib.submessages().iter().enumerate() {
        println!("{}\n{}", i, grib.describe_submessage(submessage));
    }
    Ok(())
}
