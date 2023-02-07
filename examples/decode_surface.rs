use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // This example shows how to decode values inside a surface in a GRIB2 message.
    // The example also shows how to obtain the latitude-longitude locations of grid
    // points, which are usually used in conjunction with the grid point values.

    // Take the first argument as an input file path and the second argument as a
    // surface index.
    let mut args = env::args().skip(1);
    if let (Some(file_path), Some(index), Some(subindex)) = (args.next(), args.next(), args.next())
    {
        let index: (usize, usize) = (index.parse()?, subindex.parse()?);
        decode_surface(file_path, index)
    } else {
        panic!("Usage: decode_surface <path> <index>");
    }
}

fn decode_surface<P>(path: P, message_index: (usize, usize)) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    // Open the input file in a normal way.
    let f = File::open(path)?;
    let f = BufReader::new(f);

    // Read with the reader.
    let grib2 = grib::from_reader(f)?;

    // Find the target submessage.
    let (_index, submessage) = grib2
        .iter()
        .find(|(index, _)| *index == message_index)
        .ok_or("no such index")?;

    // Obtain latitude-longitude locations as an iterator.
    let latlons = submessage.latlons()?;

    // Prepare a decoder.
    let decoder = grib::Grib2SubmessageDecoder::from(submessage)?;

    // Actually dispatch a decoding process and get an iterator of decoded values.
    // There are various methods available for compressing GRIB2 data, but some are
    // not yet supported by this library and may return errors.
    let values = decoder.dispatch()?;

    // Iterate over decoded values along with locations.
    for ((lat, lon), value) in latlons.zip(values) {
        println!("{lat} {lon} {value}");
    }

    Ok(())
}
