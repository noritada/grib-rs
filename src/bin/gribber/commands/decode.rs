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

pub fn exec(args: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = args.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;
    let index: usize = args.value_of("index").unwrap().parse()?;
    let values = grib.get_values(index)?;

    if args.is_present("big-endian") {
        let out_path = args.value_of("big-endian").unwrap();
        File::create(out_path)
            .and_then(|mut f| {
                for value in values.iter() {
                    f.write_all(&value.to_be_bytes())?;
                }
                Ok(())
            })
            .map_err(|e| cli::CliError::IOError(e, out_path.to_string()))?;
    } else if args.is_present("little-endian") {
        let out_path = args.value_of("little-endian").unwrap();
        File::create(out_path)
            .and_then(|mut f| {
                for value in values.iter() {
                    f.write_all(&value.to_le_bytes())?;
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
