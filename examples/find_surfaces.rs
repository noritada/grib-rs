use grib::codetables::grib2::*;
use grib::codetables::*;
use grib::context::Grib2;
use grib::datatypes::*;
use grib::reader::SeekableGrib2Reader;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
    // This example shows how to find surfaces in a GRIB2 message.

    // Take the first argument as an input file path and the second argument as forecast time in hours.
    let mut args = env::args().skip(1);
    if let (Some(file_name), Some(forecast_time)) = (args.next(), args.next()) {
        let path = Path::new(&file_name);
        let forecast_time = forecast_time.parse::<u32>().unwrap();
        find_surfaces(path, forecast_time);
    } else {
        panic!("Usage: find_surfaces <path> <forecast_time>");
    }
}

fn find_surfaces(path: &Path, forecast_time_hours: u32) {
    // Open the input file in a normal way, ignoring errors.
    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);

    // Read with the reader provided by the library. Errors are ignored in this case, too.
    // This interface is ugly and will be improved in the future.
    let grib2 = Grib2::<SeekableGrib2Reader<BufReader<File>>>::read_with_seekable(f).unwrap();

    for (index, submessage) in grib2.iter().enumerate() {
        let ft = submessage.prod_def().forecast_time();
        match ft {
            Some(ForecastTime {
                unit: Name(Table4_4::Hour),
                value: hours,
            }) => {
                if hours == forecast_time_hours {
                    println!("{}: {}", index, hours);
                }
            }
            _ => {}
        }
    }
}
