use std::{env, error::Error, fs::File, io::BufReader, path::Path};

use grib::{codetables::grib2::*, ForecastTime, Name};

fn main() -> Result<(), Box<dyn Error>> {
    // This example shows how to find layers in a GRIB2 message.

    // Take the first argument as an input file path and the second argument as
    // forecast time in hours.
    let mut args = env::args().skip(1);
    if let (Some(file_path), Some(forecast_time)) = (args.next(), args.next()) {
        let forecast_time = forecast_time.parse::<u32>()?;
        find_layers(file_path, forecast_time)
    } else {
        panic!("Usage: find_layers <path> <forecast_time>");
    }
}

fn find_layers<P>(path: P, forecast_time_hours: u32) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    // Open the input file in a normal way.
    let f = File::open(path)?;
    let f = BufReader::new(f);

    // Read with the reader.
    let grib2 = grib::from_reader(f)?;

    for (index, submessage) in grib2.iter() {
        let ft = submessage.prod_def().forecast_time();
        if matches!(
            ft,
            Some(ForecastTime {
                unit: Name(Table4_4::Hour),
                value: hours,
            }) if hours == forecast_time_hours
        ) {
            println!("{}.{}\n{}", index.0, index.1, submessage.describe());
        }
    }

    Ok(())
}
