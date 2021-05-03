use grib::context::Grib2;
use grib::reader::SeekableGrib2Reader;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
    // This example shows how to decode values inside a surface in a GRIB2 message.
    // Note that this script simply decodes all the values inside a surface.
    // For now, there is no functionality to get which grid point each value is associated with.

    // Take the first argument as an input file path and the second argument as a surface index.
    let mut args = env::args();
    let file_name = args.nth(1).unwrap();
    let path = Path::new(&file_name);
    let index = args.next().unwrap();
    let index = index.parse::<usize>().unwrap();

    // Open the input file in a normal way, ignoring errors.
    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);

    // Read with the reader provided by the library. Errors are ignored in this case, too.
    // This interface is ugly and will be improved in the future.
    let grib2 = Grib2::<SeekableGrib2Reader<BufReader<File>>>::read_with_seekable(f).unwrap();

    // There are various methods available for compressing GRIB2 data, but some are not yet
    // supported by this library and may return errors.
    // We simply ignore the errors here.
    let values = grib2.get_values(index).unwrap();

    // Iterate over decoded values.
    for value in values.iter() {
        println!("{}", value);
    }
}
