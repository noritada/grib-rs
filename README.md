# grib-rs

[![docs](https://docs.rs/grib/badge.svg)](https://docs.rs/grib)
[![Crates.io](https://img.shields.io/crates/v/grib)](https://crates.io/crates/grib)
[![dependency status](https://deps.rs/repo/github/noritada/grib-rs/status.svg)](https://deps.rs/repo/github/noritada/grib-rs)
[![License (Apache 2.0)](https://img.shields.io/badge/license-Apache%202.0-blue)](https://github.com/noritada/grib-rs/blob/master/LICENSE-APACHE)
[![License (MIT)](https://img.shields.io/badge/license-MIT-blue)](https://github.com/noritada/grib-rs/blob/master/LICENSE-MIT)
[![Build](https://github.com/noritada/grib-rs/workflows/CI/badge.svg)](https://github.com/noritada/grib-rs/actions?query=workflow%3ACI)

GRIB format parser for Rust

## About

This is a GRIB format parser library written in [Rust](https://www.rust-lang.org/) programming language. This project aims to provide a set of library and tools which is simple-to-use, efficient, and educational.

GRIB is a concise data format commonly used in meteorology to store historical and forecast weather data. It is intended to be a container of a collection of records of 2D data. GRIB files are huge and binary and should be processed efficiently. Also, since GRIB is designed to support various grid types and data compression using parameters defined in external code tables and templates, some popular existing softwares cannot handle some GRIB data.

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

### Supported grid definition templates

For data using the following grid systems, latitudes and longitudes of grid points can be computed.

| Template number | Grid system | Notes |
| --- | --- | --- |
| 3.0 | latitude/longitude (or equidistant cylindrical, or Plate Carree) ||
| 3.20 | Polar stereographic projection | enabling feature `gridpoints-proj` required |
| 3.30 | Lambert conformal | enabling feature `gridpoints-proj` required |

### Supported data representation templates

For data using the following encoding methods, grid point values can be extracted.

| Template number | Encoding method |
| --- | --- |
| 5.0 | simple packing |
| 5.2 | complex packing |
| 5.3 | complex packing and spatial differencing |
| 5.40 | JPEG 2000 code stream format |
| 5.41 | Portable Network Graphics (PNG) |
| 5.200 | run length packing with level values |

## Planned features

Please check the [ROADMAP](ROADMAP.md) to see planned features.

## API

API Documentation of the released version of the library crate is [available on Docs.rs](https://docs.rs/grib/latest/grib/) although it is not extensive. The development version is [available on GitHub Pages](https://noritada.github.io/grib-rs/grib/index.html).

If you feel a feature is missing, please send us your suggestions through the [GitHub Issues](https://github.com/noritada/grib-rs/issues/new/choose). We are working on expanding the basic functionality as our top priority in this project, so we would be happy to receive any requests.

### Usage example

```rust
use grib::{self, codetables::grib2::*, ForecastTime, Grib2SubmessageDecoder, Name};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fname = "Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin";
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

The [examples directory](examples) may help you understand the API.

## CLI application `gribber`

CLI application `gribber` built on the top of the `grib` library is available. It is in the `grib-cli` package and can be installed via `cargo install grib-cli`.

```
Usage: gribber [COMMAND]

Commands:
  completions  Generate shell completions for your shell to stdout
  decode       Export decoded data
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

```
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
