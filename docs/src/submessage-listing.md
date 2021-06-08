# Listing all submessages inside a GRIB message

wgrib2:

```shell
wgrib2 datafile.grib
```

pygrib:

```python
import pygrib

grib = pygrib.open("datafile.grib")
for submessage in grib:
    print(submessage)
```

grib-rs:

```rust
use grib::codetables::{CodeTable4_2, Lookup};
use grib::context::Grib2;
use grib::reader::SeekableGrib2Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn list_submessages() {
    let path = Path::new("datafile.grib");
    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);

    let grib2 = Grib2::<SeekableGrib2Reader<BufReader<File>>>::read_with_seekable(f).unwrap();

    for submessage in grib2.iter() {
        let discipline = submessage.indicator().discipline;
        let category = submessage.prod_def().parameter_category().unwrap();
        let parameter = submessage.prod_def().parameter_number().unwrap();
        let parameter = CodeTable4_2::new(discipline, category).lookup(usize::from(parameter));

        let forecast_time = submessage.prod_def().forecast_time().unwrap();
        let (unit, forecast_time) = forecast_time.describe();

        let (first, _second) = submessage.prod_def().fixed_surfaces().unwrap();
        let elevation_level = first.value();

        println!(
            "{}\t\t{} {}\t{}",
            parameter, forecast_time, unit, elevation_level
        );
    }
}

fn main() {
    list_submessages();
}
```

gribber:

```shell
gribber list datafile.grib
```
