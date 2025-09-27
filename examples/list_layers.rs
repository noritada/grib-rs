use std::{env, error::Error, fs::File, io::BufReader, path::Path};

use grib::codetables::{CodeTable4_2, Lookup};

fn main() -> Result<(), Box<dyn Error>> {
    // This example shows how to get information of element names, forecast time and
    // elevation levels for all layers in a GRIB2 message.

    // Take the first argument as an input file path.
    let mut args = env::args().skip(1);
    if let Some(file_path) = args.next() {
        list_layers(file_path)
    } else {
        panic!("Usage: list_layers <path>");
    }
}

fn list_layers<P>(path: P) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    // Open the input file in a normal way.
    let f = File::open(path)?;
    let f = BufReader::new(f);

    // Read with the reader.
    let grib2 = grib::from_reader(f)?;

    // Iterate over layers.
    for (_index, submessage) in grib2.iter() {
        // In GRIB data, attribute information such as elements are represented as
        // numeric values. To convert those numeric values to strings, we use
        // tables called Code Tables. Code Table 4.2 is required for the textual
        // representation of element names, and Code Table 4.3 is required for
        // the textual representation of forecast time units.
        //
        // Code Table 4.2 does not actually give you a unique text if you just specify a
        // number. It has a hierarchical structure in which multiple product
        // disciplines are defined, each containing multiple parameter
        // categories, and many parameters within each category. For example,
        // the product discipline includes meteorological products (0), hydrological
        // products (1), etc. and in the meteorological products, there are temperature
        // (0), moisture (1), momentum (2), etc.
        // The momentum includes wind direction (0), wind speed (1), u-component of wind
        // (2), v-component of wind (3), and so on.
        //
        // Therefore, it would be easy to just get the numerical representation of the
        // elements, but since we want to display the element names here, we
        // also need to get the product discipline and parameter category, and
        // then convert the parameter number using Code Table 4.2.

        // Product discipline is included in the indicator section (and common in a
        // GRIB2 message).
        let discipline = submessage.indicator().discipline;
        // Parameter category and number are included in the product definition section.
        // They are wrapped by `Option` because some GRIB2 data may not contain such
        // information.
        let category = submessage.prod_def().parameter_category().unwrap();
        let parameter = submessage.prod_def().parameter_number().unwrap();

        // When using the `lookup()` function, `use grib::codetables::Lookup;` is
        // necessary.
        let parameter = CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));

        // `forecast_time()` returns `ForecastTime` wrapped by `Option`.
        let forecast_time = submessage.prod_def().forecast_time().unwrap();

        // `fixed_layers()` returns a tuple of two layers wrapped by `Option`.
        let (first, _second) = submessage.prod_def().fixed_surfaces().unwrap();
        let elevation_level = first.value();
        let elevation_unit = first.unit().map(|s| format!(" [{s}]")).unwrap_or_default();

        println!(
            "{:<31} {:>14} {:>17}{}",
            parameter.to_string(),
            forecast_time,
            elevation_level,
            elevation_unit,
        );
    }

    Ok(())
}
