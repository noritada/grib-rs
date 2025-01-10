use std::{fmt, path::PathBuf};

use anyhow::Result;
use clap::{arg, ArgMatches, Command};
use console::Style;
use grib::GribError;

use crate::cli;

pub fn cli() -> Command {
    Command::new(crate::cli::module_component!())
        .about("Export decoded data with latitudes and longitudes")
        .arg(
            arg!(<FILE> "Target file name (or a single dash (`-`) for standard input)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(arg!(<INDEX> "Submessage index"))
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
    let index = args.get_one::<String>("INDEX").unwrap();
    let cli::CliMessageIndex(message_index) = index.parse()?;
    let (_, submessage) = grib
        .iter()
        .find(|(index, _)| *index == message_index)
        .ok_or_else(|| anyhow::anyhow!("no such index: {}.{}", message_index.0, message_index.1))?;
    let latlons = submessage.latlons();
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;
    let values = decoder.dispatch()?;

    if args.contains_id("big-endian") {
        let out_path = args.get_one::<PathBuf>("big-endian").unwrap();
        write_output(out_path, values, |f| f.to_be_bytes())
    } else if args.contains_id("little-endian") {
        let out_path = args.get_one::<PathBuf>("little-endian").unwrap();
        write_output(out_path, values, |f| f.to_le_bytes())
    } else {
        let values = values.collect::<Vec<_>>().into_iter(); // workaround for mutability
        let latlons = match latlons {
            Ok(iter) => LatLonIteratorWrapper::LatLon(iter),
            Err(GribError::NotSupported(_)) => {
                let nan_iter = vec![(f32::NAN, f32::NAN); values.len()].into_iter();
                LatLonIteratorWrapper::NaN(nan_iter)
            }
            Err(e) => anyhow::bail!("something unexpected happened:: {e}"),
        };
        let values = latlons.zip(values);
        cli::display_in_pager(DecodeTextDisplay(values));
        Ok(())
    }
}

#[derive(Clone)]
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

struct DecodeTextDisplay<I>(I);

impl<I> cli::PredictableNumLines for DecodeTextDisplay<I>
where
    I: Iterator<Item = ((f32, f32), f32)>,
{
    fn num_lines(&self) -> usize {
        let Self(inner) = self;
        let (len, _) = inner.size_hint();
        len + 1
    }
}

impl<I> fmt::Display for DecodeTextDisplay<I>
where
    I: Iterator<Item = ((f32, f32), f32)> + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = format!("{:>10} {:>11} {:>9}", "Latitude", "Longitude", "Value",);
        let style = Style::new().bold();
        writeln!(f, "{}", style.apply_to(header.trim_end()))?;

        let Self(inner) = self;
        // cloning just to work around a mutability issue
        for ((lat, lon), value) in inner.clone() {
            // lat/lons are formatted in "-?\d{2}.\d{6} -?\d{2}.\d{6}"
            writeln!(f, "{lat:>10.6} {lon:>11.6} {value:>9}")?;
        }
        Ok(())
    }
}
