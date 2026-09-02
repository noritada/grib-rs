use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use anyhow::Result;
use clap::{ArgMatches, Command, arg};
use console::Style;
use grib::{GribError, LatLons};

use crate::cli;

pub fn cli() -> Command {
    Command::new(crate::cli::module_component!())
        .about("Export decoded data with latitudes and longitudes")
        .arg(
            arg!(<FILE> "Target file name (or a single dash (`-`) for standard input)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(arg!(<INDEX> "Submessage index").value_parser(clap::value_parser!(cli::CliMessageIndex)))
        .arg(
            arg!(-b --"big-endian" <OUT_FILE> "Export (without lat/lon) as a big-endian flat binary file")
                .required(false) // There is no syntax yet for optional options.
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            arg!(-l --"little-endian" <OUT_FILE> "Export (without lat/lon) as a little-endian flat binary file")
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
    let mut stream = crate::cli::WriteStream::new(out_path)?;
    values.try_for_each(|f| stream.write_all(&to_bytes(&f)))?;
    Ok(())
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;
    let cli::CliMessageIndex(message_index) = args.get_one("INDEX").unwrap();
    let (_, submessage) = grib
        .iter()
        .find(|(index, _)| index == message_index)
        .ok_or_else(|| anyhow::anyhow!("no such index: {}.{}", message_index.0, message_index.1))?;
    let latlons = if args.contains_id("big-endian") || args.contains_id("little-endian") {
        None
    } else {
        Some(submessage.latlons())
    };
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;
    let values = decoder.dispatch()?;

    if args.contains_id("big-endian") {
        let out_path = args.get_one::<PathBuf>("big-endian").unwrap();
        write_output(out_path, values, |f| f.to_be_bytes())
    } else if args.contains_id("little-endian") {
        let out_path = args.get_one::<PathBuf>("little-endian").unwrap();
        write_output(out_path, values, |f| f.to_le_bytes())
    } else {
        let num_values = values.size_hint().0;
        let latlons = match latlons.expect("lat/lon result is present for text output") {
            Ok(iter) => LatLonIteratorWrapper::LatLon(iter),
            Err(GribError::NotSupported(_)) => {
                let nan_iter = std::iter::repeat_n((f32::NAN, f32::NAN), num_values);
                LatLonIteratorWrapper::NaN(nan_iter)
            }
            Err(e) => anyhow::bail!("something unexpected happened:: {e}"),
        };
        write_text_output(latlons.zip(values))
    }
}

enum LatLonIteratorWrapper<L, N> {
    LatLon(L),
    NaN(N),
}

impl<L, N> Iterator for LatLonIteratorWrapper<L, N>
where
    L: Iterator<Item = (f32, f32)>,
    N: Iterator<Item = (f32, f32)>,
{
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::LatLon(value) => value.next(),
            Self::NaN(value) => value.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::LatLon(value) => value.size_hint(),
            Self::NaN(value) => value.size_hint(),
        }
    }
}

fn write_text_output(values: impl Iterator<Item = ((f32, f32), f32)>) -> Result<()>
where
{
    let num_lines = values.size_hint().0 + 1;
    cli::prepare_pager(num_lines);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let header = format!("{:>10} {:>11} {:>9}", "Latitude", "Longitude", "Value",);
    let style = Style::new().bold();
    writeln!(out, "{}", style.apply_to(header.trim_end()))?;

    for ((lat, lon), value) in values {
        // lat/lons are formatted in "-?\d{2}.\d{6} -?\d{2}.\d{6}"
        writeln!(out, "{lat:>10.6} {lon:>11.6} {value:>9}")?;
    }

    Ok(())
}
