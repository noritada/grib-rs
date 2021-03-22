# grib-rs

[![docs](https://docs.rs/grib/badge.svg)](https://docs.rs/grib)
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
  * Support for Common Code Table C-0 and C-11
  * Decoding feature supporting following templates:
    * Template 5.0/7.0 (simple packing)
    * Template 5.3/7.3 (complex packing)
    * Template 5.200/7.200 (run-length encoding)
* CLI application `gribber` built on the top of the Rust library
  * 4 subcommends:
    * decode: data export as text and flat binary files
    * info: display of identification information
    * inspect: display of information mainly for development purpose such as template numbers
    * list: display of a list of sections (the style is still tentative)

## Planned features

* WebAssembly application
* Efficient read from cloud sources such as S3
* More supports of code tables and templates
* Data extraction based on geographical specification
* Format conversion to other popular formats
* Providing interface to other languages

## CLI application `gribber`

```
USAGE:
    gribber [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    decode     Exports decoded data
    help       Prints this message or the help of the given subcommand(s)
    info       Shows identification information
    inspect    Inspects and describes the data structure
    list       Lists contained data
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
