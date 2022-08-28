use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // This example shows how to decode values inside a surface in a GRIB2 message.
    // Note that this script simply decodes all the values inside a surface.
    // For now, there is no functionality to get which grid point each value is
    // associated with.

    // Take the first argument as an input file path and the second argument as a
    // surface index.
    let mut args = env::args().skip(1);
    if let (Some(file_path), Some(index), Some(subindex)) = (args.next(), args.next(), args.next())
    {
        let index: (usize, usize) = (index.parse()?, subindex.parse()?);
        decode_surface(&file_path, index)
    } else {
        panic!("Usage: decode_surface <path> <index>");
    }
}

fn decode_surface<P>(path: P, index: (usize, usize)) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    // Open the input file in a normal way.
    let f = File::open(path)?;
    let f = BufReader::new(f);

    // Read with the reader.
    let grib2 = grib::from_reader(f)?;

    // There are various methods available for compressing GRIB2 data, but some are
    // not yet supported by this library and may return errors.
    let values = grib2.get_values(index)?;

    // Iterate over decoded values.
    for value in values.iter() {
        println!("{}", value);
    }

    Ok(())
}
