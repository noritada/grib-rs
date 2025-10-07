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
  * 5 subcommends:
    * completions: generation of shell completions for your shell
    * decode: data export as text and flat binary files
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

### Usage example

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
