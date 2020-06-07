use clap::{App, Arg, ArgMatches, SubCommand};
use std::fs::File;
use std::io::Write;

use crate::cli;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("decode")
        .about("Exports decoded data")
        .arg(Arg::with_name("file").required(true))
        .arg(Arg::with_name("index").required(true))
        .arg(
            Arg::with_name("big-endian")
                .help("Exports as a big-endian flat binary file")
                .short("b")
                .long("big-endian")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("little-endian")
                .help("Exports as a little-endian flat binary file")
                .short("l")
                .long("little-endian")
                .takes_value(true)
                .conflicts_with("big-endian"),
        )
}

pub fn exec(subcommand_matches: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = subcommand_matches.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;
    let index: usize = subcommand_matches.value_of("index").unwrap().parse()?;
    let values = grib.get_values(index)?;

    if subcommand_matches.is_present("big-endian") {
        let out_path = subcommand_matches.value_of("big-endian").unwrap();
        File::create(out_path)
            .and_then(|mut f| {
                for value in values.iter() {
                    f.write(&value.to_be_bytes())?;
                }
                Ok(())
            })
            .map_err(|e| cli::CliError::IOError(e, out_path.to_string()))?;
    } else if subcommand_matches.is_present("little-endian") {
        let out_path = subcommand_matches.value_of("little-endian").unwrap();
        File::create(out_path)
            .and_then(|mut f| {
                for value in values.iter() {
                    f.write(&value.to_le_bytes())?;
                }
                Ok(())
            })
            .map_err(|e| cli::CliError::IOError(e, out_path.to_string()))?;
    } else {
        cli::start_pager();
        println!("{:#?}", values);
    }

    Ok(())
}
