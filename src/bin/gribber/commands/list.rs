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
    for (i, _submessage) in grib.submessages().iter().enumerate() {
        if let Some(submessage) = grib.get_submessage(i) {
            println!("{}\n{}", i, submessage.describe());
        }
    }
    Ok(())
}
