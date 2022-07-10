use grib::codetables::grib2::*;
use grib::codetables::*;
use grib::datatypes::*;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // This example shows how to find surfaces in a GRIB2 message.

    // Take the first argument as an input file path and the second argument as
    // forecast time in hours.
    let mut args = env::args().skip(1);
    if let (Some(file_name), Some(forecast_time)) = (args.next(), args.next()) {
        let path = Path::new(&file_name);
        let forecast_time = forecast_time.parse::<u32>()?;
        find_surfaces(path, forecast_time)
    } else {
        panic!("Usage: find_surfaces <path> <forecast_time>");
    }
}

fn find_surfaces(path: &Path, forecast_time_hours: u32) -> Result<(), Box<dyn Error>> {
    // Open the input file in a normal way.
    let f = File::open(&path)?;
    let f = BufReader::new(f);

    // Read with the reader.
    let grib2 = grib::from_reader(f)?;

    for (index, submessage) in grib2.iter().enumerate() {
        let ft = submessage.prod_def().forecast_time();
        if let Some(ForecastTime {
            unit: Name(Table4_4::Hour),
            value: hours,
        }) = ft
        {
            if hours == forecast_time_hours {
                println!("{}: {}", index, hours);
            }
        }
    }

    Ok(())
}
