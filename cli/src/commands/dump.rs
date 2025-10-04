use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgMatches, Command, arg};

use crate::cli;

pub fn cli() -> Command {
    Command::new(crate::cli::module_component!())
        .about("Dump the content of a GRIB submessage")
        .arg(
            arg!(<FILE> "Target file name (or a single dash (`-`) for standard input)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            arg!(<INDEX> "Submessage index")
                .value_parser(clap::value_parser!(cli::CliMessageIndex)),
        )
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;
    let cli::CliMessageIndex(message_index) = args.get_one("INDEX").unwrap();
    let (_, submessage) = grib
        .iter()
        .find(|(index, _)| index == message_index)
        .ok_or_else(|| anyhow::anyhow!("no such index: {}.{}", message_index.0, message_index.1))?;
    let mut stream = std::io::stdout();
    submessage.dump(&mut stream)?;
    Ok(())
}
