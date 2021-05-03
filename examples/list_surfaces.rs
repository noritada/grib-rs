use grib::codetables::{CodeTable4_2, CodeTable4_4, Lookup};
use grib::context::Grib2;
use grib::reader::SeekableGrib2Reader;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
    // This example shows how to get information of element names and forecast time for all
    // surfaces in a GRIB2 message.

    // Take the first argument as an input file path.
    let mut args = env::args();
    let file_name = args.nth(1).unwrap();
    let path = Path::new(&file_name);

    // Open the input file in a normal way, ignoring errors.
    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);

    // Read with the reader provided by the library. Errors are ignored in this case, too.
    // This interface is ugly and will be improved in the future.
    let grib2 = Grib2::<SeekableGrib2Reader<BufReader<File>>>::read_with_seekable(f).unwrap();

    // Iterate over surfaces.
    for submessage in grib2.submessages() {
        // In GRIB data, attribute information such as elements are represented as numeric values.
        // To convert those numeric values to strings, we use tables called Code Tables.
        // Code Table 4.2 is required for the textual representation of element names, and Code
        // Table 4.3 is required for the textual representation of forecast time units.
        //
        // Code Table 4.2 does not actually give you a unique text if you just specify a number.
        // It has a hierarchical structure in which multiple product disciplines are defined, each
        // containing multiple parameter categories, and many parameters within each category.
        // For example, the product discipline includes meteorological products (0), hydrological
        // products (1), etc. and in the meteorological products, there are temperature (0),
        // moisture (1), momentum (2), etc.
        // The momentum includes wind direction (0), wind speed (1), u-component of wind (2),
        // v-component of wind (3), and so on.
        //
        // Therefore, it would be easy to just get the numerical representation of the elements,
        // but since we want to display the element names here, we also need to get the product
        // discipline and parameter category, and then convert the parameter number using Code
        // Table 4.2.

        // Product discipline is included in the indicator section (and common in a GRIB2 message).
        let discipline = submessage.indicator().discipline;
        // Parameter category and number are included in the product definition section.
        // They are wrapped by `Option` because some GRIB2 data may not contain such information.
        let category = submessage.prod_def().parameter_category().unwrap();
        let parameter = submessage.prod_def().parameter_number().unwrap();

        // When using the `lookup()` function, `use grib::codetables::Lookup;` is necessary.
        let parameter = CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));

        // `forecast_time()` returns a tuple wrapped by `Option`.
        let (unit, forecast_time) = submessage.prod_def().forecast_time().unwrap();
        let unit = CodeTable4_4.lookup(usize::from(unit));

        println!("{}\t\t{} {}", parameter, forecast_time, unit);
    }
}
