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
    * List of surfaces
    * Some parameters of each surface, which are important for most users
    * Underlying sections which make up surfaces and the entire data
  * Support for some code tables defined by WMO
  * Decoding feature supporting following templates:
    * Template 5.0/7.0 (simple packing)
    * Template 5.3/7.3 (complex packing and spatial differencing)
    * Template 5.40/7.40 (JPEG 2000 code stream format)
    * Template 5.200/7.200 (run-length encoding)
* CLI application `gribber` built on the top of the Rust library
  * 4 subcommends:
    * decode: data export as text and flat binary files
    * info: display of identification information
    * inspect: display of information mainly for development purpose such as template numbers
    * list: display of parameters for each surface inside

## Planned features

Please check the [ROADMAP](ROADMAP.md) to see planned features.

## API

API Documentation of the released version of the library crate is [available on Docs.rs](https://docs.rs/grib/latest/grib/) although it is not extensive. The development version is [available on GitHub Pages](https://noritada.github.io/grib-rs/grib/index.html).

The [examples](examples) may help you understand the API. The functionality is still inadequate, and we are working on expanding the basic functionality as our top priority in this project, so we would be happy to receive any requests.

## CLI application `gribber`

```
Usage: gribber [COMMAND]

Commands:
  decode   Export decoded data
  info     Show identification information
  inspect  Inspect and describes the data structure
  list     List surfaces contained in the data
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
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
