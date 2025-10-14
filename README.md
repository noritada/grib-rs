# grib-rs

[![docs](https://docs.rs/grib/badge.svg)](https://docs.rs/grib)
[![Crates.io](https://img.shields.io/crates/v/grib)](https://crates.io/crates/grib)
[![dependency status](https://deps.rs/repo/github/noritada/grib-rs/status.svg)](https://deps.rs/repo/github/noritada/grib-rs)
[![License (Apache 2.0)](https://img.shields.io/badge/license-Apache%202.0-blue)](https://github.com/noritada/grib-rs/blob/master/LICENSE-APACHE)
[![License (MIT)](https://img.shields.io/badge/license-MIT-blue)](https://github.com/noritada/grib-rs/blob/master/LICENSE-MIT)
[![Build](https://github.com/noritada/grib-rs/workflows/CI/badge.svg)](https://github.com/noritada/grib-rs/actions?query=workflow%3ACI)

GRIB format parser for Rust

## About

This is a GRIB format parser library written in Rust programming language. This project aims to provide a set of library and tools which is simple-to-use, efficient, and educational.

GRIB is a concise data format commonly used in meteorology to store historical and forecast weather data. It is intended to be a container of a collection of records of 2D data. GRIB files are huge and binary and should be processed efficiently. Also, since GRIB is designed to support various grid types and data compression using parameters defined in external code tables and templates, some popular existing softwares cannot handle some GRIB data.

## Demo web application

GRIB2 viewer web app for demo using the crate is available [here](https://noritada.github.io/grib-rs/viewer/).

After loaded, the app works completely on your web browser and will not send the data you drop anywhere.

## Vision

A world where everyone can read weather data easily although its interpretation needs some specific knowledge and experience.

## Features

* Rust library `grib`
  * Ability to read and check the basic structure of GRIB2
  * Ability to access data inside the GRIB2 message:
    * List of layers
    * Some parameters of each layer, which are important for most users
    * Underlying sections which make up layers and the entire data
  * Support for some code tables defined by WMO
  * Decoding feature supporting templates listed in the following table
  * Support for computation of latitudes and longitudes of grid points for templates listed in the following table
* CLI application `gribber` built on the top of the Rust library
  * 6 subcommends:
    * completions: generation of shell completions for your shell
    * decode: data export as text and flat binary files
    * dump: dump display of the content of a GRIB submessage
    * info: display of identification information
    * inspect: display of information mainly for development purpose such as template numbers
    * list: display of parameters for each layer inside

### Template support

GRIB2 can contain grid point values for various grid systems. This diversity is supported by a mechanism called "templates".

Although GRIB2 contains a large number of grid point values, the coordinates and values of individual grid points are not encoded directly as numerical data. Since the grid points are regularly arranged, the coordinates can be defined by the type of projection method used for the grid system and the specific parameters for that projection method, so only a simple definition of the grid system is encoded in the data.

Also, since the best encoding method for values varies from data to data, there are multiple methods that can be used to encode values, and the method used and the specific parameters needed to encode it are defined along with the data itself.

These definitions of grid systems and data representation are represented by sequences of bytes called templates, which should be supported in order for the reader to read GRIB2 data. grib-rs supports the following templates. We would love to support other templates as well, so please let us know if there is any data that is not readable.

#### Supported grid definition templates

For data using the following grid systems, latitudes and longitudes of grid points can be computed.

| Template number | Grid system | Notes |
| --- | --- | --- |
| 3.0 | latitude/longitude (or equidistant cylindrical, or Plate Carree) | supporting only regular grids |
| 3.20 | Polar stereographic projection | enabling feature `gridpoints-proj` required |
| 3.30 | Lambert conformal | enabling feature `gridpoints-proj` required |
| 3.40 | Gaussian latitude/longitude | supporting only regular grids |

#### Supported data representation templates

For data using the following encoding methods, grid point values can be extracted.

| Template number | Encoding method | Notes |
| --- | --- | --- |
| 5.0 | simple packing ||
| 5.2 | complex packing ||
| 5.3 | complex packing and spatial differencing ||
| 5.40 | JPEG 2000 code stream format | enabling feature `jpeg2000-unpack-with-openjpeg` required |
| 5.41 | Portable Network Graphics (PNG) | enabling feature `png-unpack-with-png-crate` required |
| 5.42 | CCSDS recommended lossless compression | enabling feature `ccsds-unpack-with-libaec` required |
| 5.200 | run length packing with level values ||

## Planned features

Please check the [ROADMAP](ROADMAP.md) to see planned features.

## API

API Documentation of the released version of the library crate is [available on Docs.rs](https://docs.rs/grib/latest/grib/) although it is not extensive. The development version is [available on GitHub Pages](https://noritada.github.io/grib-rs/grib/index.html).

If you feel a feature is missing, please send us your suggestions through the [GitHub Issues](https://github.com/noritada/grib-rs/issues/new/choose). We are working on expanding the basic functionality as our top priority in this project, so we would be happy to receive any requests.

### Usage examples

This is an example of accessing the grid point values and latitude/longitude coordinates for a specific submessage within a GRIB2 file.

```rust
use grib::{codetables::grib2::*, ForecastTime, Grib2SubmessageDecoder, Name};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fname = "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin";
    let f = std::fs::File::open(fname)?;
    let f = std::io::BufReader::new(f);
    let grib2 = grib::from_reader(f)?;

    let (_index, submessage) = grib2
        .iter()
        .find(|(_index, submessage)| {
            matches!(
                submessage.prod_def().forecast_time(),
                Some(ForecastTime {
                    unit: Name(Table4_4::Minute),
                    value: minutes,
                }) if minutes == 30
            )
        })
        .ok_or("message with FT being 30 minutes not found")?;

    let latlons = submessage.latlons()?;
    let decoder = Grib2SubmessageDecoder::from(submessage)?;
    let values = decoder.dispatch()?;

    for ((lat, lon), value) in latlons.zip(values) {
        println!("{lat} {lon} {value}");
    }

    Ok(())
}
```

This is an example of accessing the parameter values contained within a section and template parameters of a specific submessage in a GRIB2 file.

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fname = "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin";
    let f = std::fs::File::open(fname)?;
    let f = std::io::BufReader::new(f);
    let grib2 = grib::from_reader(f)?;
    let (_index, first_submessage) = grib2.iter().next().unwrap();

    let actual = first_submessage.section5();
    let expected = Ok(grib::def::grib2::Section5 {
        header: grib::def::grib2::SectionHeader {
            len: 23,
            sect_num: 5,
        },
        payload: grib::def::grib2::Section5Payload {
            num_encoded_points: 86016,
            template_num: 200,
            template: grib::def::grib2::DataRepresentationTemplate::_5_200(
                grib::def::grib2::template::Template5_200 {
                    num_bits: 8,
                    max_val: 3,
                    max_level: 3,
                    dec: 0,
                    level_vals: vec![1, 2, 3],
                },
            ),
        },
    });
    assert_eq!(actual, expected);

    Ok(())
}
```

You can also dump parameters contained within submessages.
Currently, this is only implemented for certain sections.
Below is an example of dumping the same submessage as in the previous example.

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fname = "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin";
    let f = std::fs::File::open(fname)?;
    let f = std::io::BufReader::new(f);
    let grib2 = grib::from_reader(f)?;
    let (_index, first_submessage) = grib2.iter().next().unwrap();

    // In the following example, the dump output is written to a buffer for testing purposes.
    // To dump to standard output, use the following instead:
    //
    //     first_submessage.dump(std::io::stdout())?;
    let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
    first_submessage.dump(&mut buf)?;
    let expected = "\
##  SUBMESSAGE (total_length = 10321)
###  SECTION 0: INDICATOR SECTION (length = 16)
###  SECTION 1: IDENTIFICATION SECTION (length = 21)
1-4       header.len = 21  // Length of section in octets (nn).
5         header.sect_num = 1  // Number of section.
6-7       payload.centre_id = 34  // Identification of originating/generating centre (see Common Code table C–11).
8-9       payload.subcentre_id = 0  // Identification of originating/generating subcentre (allocated by originating/generating centre).
10        payload.master_table_version = 5  // GRIB master table version number (see Common Code table C–0 and Note 1).
11        payload.local_table_version = 1  // Version number of GRIB Local tables used to augment Master tables (see Code table 1.1 and Note 2).
12        payload.ref_time_significance = 0  // Significance of reference time (see Code table 1.2).
13-14     payload.ref_time.year = 2016  // Year (4 digits).
15        payload.ref_time.month = 8  // Month.
16        payload.ref_time.day = 22  // Day.
17        payload.ref_time.hour = 2  // Hour.
18        payload.ref_time.minute = 0  // Minute.
19        payload.ref_time.second = 0  // Second.
20        payload.prod_status = 0  // Production status of processed data in this GRIB message (see Code table 1.3).
21        payload.data_type = 2  // Type of processed data in this GRIB message (see Code table 1.4).
###  SECTION 3: GRID DEFINITION SECTION (length = 72)
###  SECTION 4: PRODUCT DEFINITION SECTION (length = 34)
###  SECTION 5: DATA REPRESENTATION SECTION (length = 23)
1-4       header.len = 23  // Length of section in octets (nn).
5         header.sect_num = 5  // Number of section.
6-9       payload.num_encoded_points = 86016  // Number of data points where one or more values are specified in Section 7 when a bit map is present, total number of data points when a bit map is absent.
10-11     payload.template_num = 200  // Data representation template number (see Code table 5.0).
12        payload.template.num_bits = 8  // Number of bits used for each packed value in the run length packing with level value.
13-14     payload.template.max_val = 3  // MV - maximum value within the levels that are used in the packing.
15-16     payload.template.max_level = 3  // MVL - maximum value of level (predefined).
17        payload.template.dec = 0  // Decimal scale factor of representative value of each level.
18-23     payload.template.level_vals = [1, 2, 3]  // List of MVL scaled representative values of each level from lv=1 to MVL.
###  SECTION 6: BIT-MAP SECTION (length = 6)
###  SECTION 7: DATA SECTION (length = 1391)
###  SECTION 8: END SECTION (length = 4)
";
    assert_eq!(String::from_utf8_lossy(buf.get_ref()), expected);

    Ok(())
}
```

The [examples directory][examples] in the source repository may help you understand the API.

[examples]: https://github.com/noritada/grib-rs/tree/master/examples

### Cargo features

Practical applications of GRIB data may require time calculations, processing of geospatial coordinates, and decompression of data compressed by various algorithms, which may require other libraries. Since it is not always in the best interest of all users to have dependencies on all libraries enabled at all times, this library crate uses Cargo features to make some features and dependencies optional.

See the `features` section of [Cargo.toml](Cargo.toml) for more information.

## CLI application `gribber`

CLI application `gribber` built on the top of the `grib` library is available. It is in the `grib-cli` package and can be installed via `cargo install grib-cli`.

```text
Usage: gribber [COMMAND]

Commands:
  completions  Generate shell completions for your shell to stdout
  decode       Export decoded data with latitudes and longitudes
  dump         Dump the content of a GRIB submessage
  info         Show identification information
  inspect      Inspect and describes the data structure
  list         List layers contained in the data
  help         Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Note that binaries exported from `gribber decode --big-endian` use `0x7fc00000` as a missing value, although those from `wgrib` use `0x6258d19a`.

## Building

This repository uses the submodules functionality of Git. So, before running `cargo build`, please add submodules in one of following ways:

* Cloning with submodules:
  adding `--recursive` to `git clone` will automatically clone submodules in addition to this repository
* Adding submodules after cloning:
  running `git submodule update --init --recursive` after cloning will update the repository to have submodules

Then you can build it in the usual way in the Rust world.

```shell
cargo build
```

## Forum

If you have questions or want to have discussions, feel free to use [GitHub Discussions](https://github.com/noritada/grib-rs/discussions) as a forum.

## Contributing

Contribution is always welcome.  Please check [CONTRIBUTING.md](CONTRIBUTING.md) if you are interested.

## License

This project is licensed under either of

 * Apache License, Version 2.0 (See [LICENSE-APACHE](LICENSE-APACHE)
   or http://www.apache.org/licenses/LICENSE-2.0), and
 * MIT license (See [LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

`SPDX-License-Identifier: Apache-2.0 OR MIT`
