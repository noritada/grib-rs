use clap::{arg, ArgMatches, Command};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::cli;

pub fn cli() -> Command<'static> {
    Command::new("decode")
        .about("Export decoded data")
        .arg(arg!(<FILE> "Target file").value_parser(clap::value_parser!(PathBuf)))
        .arg(arg!(<INDEX> "Submessage index").value_parser(clap::value_parser!(usize)))
        .arg(
            arg!(-b --"big-endian" <OUT_FILE> "Export as a big-endian flat binary file")
                .required(false) // There is no syntax yet for optional options.
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            arg!(-l --"little-endian" <OUT_FILE> "Export as a little-endian flat binary file")
                .required(false) // There is no syntax yet for optional options.
                .value_parser(clap::value_parser!(PathBuf))
                .conflicts_with("big-endian"),
        )
}

pub fn exec(args: &ArgMatches) -> Result<(), cli::CliError> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;
    let index = args.get_one::<usize>("INDEX").unwrap();
    let values = grib.get_values(*index)?;

    if args.contains_id("big-endian") {
        let out_path = args.get_one::<PathBuf>("big-endian").unwrap();
        File::create(out_path)
            .and_then(|f| {
                let mut stream = BufWriter::new(f);
                for value in values.iter() {
                    stream.write_all(&value.to_be_bytes())?;
                }
                Ok(())
            })
            .map_err(|e| cli::CliError::IO(e, out_path.to_string_lossy().to_string()))?;
    } else if args.contains_id("little-endian") {
        let out_path = args.get_one::<PathBuf>("little-endian").unwrap();
        File::create(out_path)
            .and_then(|f| {
                let mut stream = BufWriter::new(f);
                for value in values.iter() {
                    stream.write_all(&value.to_le_bytes())?;
                }
                Ok(())
            })
            .map_err(|e| cli::CliError::IO(e, out_path.to_string_lossy().to_string()))?;
    } else {
        cli::start_pager();
        println!("{:#?}", values);
    }

    Ok(())
}
