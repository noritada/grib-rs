use anyhow::Result;
use clap::{arg, ArgMatches, Command};
use std::fmt;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use crate::cli;

pub fn cli() -> Command {
    Command::new("decode")
        .about("Export decoded data")
        .arg(arg!(<FILE> "Target file").value_parser(clap::value_parser!(PathBuf)))
        .arg(arg!(<INDEX> "Submessage index"))
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

fn write_output(
    out_path: &PathBuf,
    mut values: impl Iterator<Item = f32>,
    to_bytes: fn(&f32) -> [u8; 4],
) -> Result<()> {
    File::create(out_path).and_then(|f| {
        let mut stream = BufWriter::new(f);
        values.try_for_each(|f| stream.write_all(&to_bytes(&f)))
    })?;
    Ok(())
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;
    let index = args.get_one::<String>("INDEX").unwrap();
    let cli::CliMessageIndex(message_index) = index.parse()?;
    let (_, submessage) = grib
        .iter()
        .find(|(index, _)| *index == message_index)
        .ok_or_else(|| anyhow::anyhow!("no such index: {}.{}", message_index.0, message_index.1))?;
    let decoder = grib::decoders::Grib2SubmessageDecoder::from(submessage)?;
    let values = decoder.dispatch()?;

    if args.contains_id("big-endian") {
        let out_path = args.get_one::<PathBuf>("big-endian").unwrap();
        write_output(out_path, values, |f| f.to_be_bytes())
    } else if args.contains_id("little-endian") {
        let out_path = args.get_one::<PathBuf>("little-endian").unwrap();
        write_output(out_path, values, |f| f.to_le_bytes())
    } else {
        cli::display_in_pager(DecodeTextDisplay(&values.collect::<Vec<_>>()));
        Ok(())
    }
}

struct DecodeTextDisplay<'a>(&'a [f32]);

impl<'a> cli::PredictableNumLines for DecodeTextDisplay<'a> {
    fn num_lines(&self) -> usize {
        let Self(inner) = self;
        inner.len()
    }
}

impl<'i> fmt::Display for DecodeTextDisplay<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self(inner) = self;
        writeln!(f, "{:#?}", inner)?;
        Ok(())
    }
}
