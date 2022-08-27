use anyhow::Result;
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

fn write_output(out_path: &PathBuf, values: &[f32], to_bytes: fn(&f32) -> [u8; 4]) -> Result<()> {
    let _ = File::create(out_path).and_then(|f| {
        let mut stream = BufWriter::new(f);
        values
            .iter()
            .try_for_each(|f| stream.write_all(&to_bytes(f)))
    })?;
    Ok(())
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;
    let index = args.get_one::<usize>("INDEX").unwrap();
    let values = grib.get_values(*index)?;

    if args.contains_id("big-endian") {
        let out_path = args.get_one::<PathBuf>("big-endian").unwrap();
        write_output(out_path, &values, |f| f.to_be_bytes())
    } else if args.contains_id("little-endian") {
        let out_path = args.get_one::<PathBuf>("little-endian").unwrap();
        write_output(out_path, &values, |f| f.to_le_bytes())
    } else {
        cli::start_pager();
        println!("{:#?}", values);
        Ok(())
    }
}
