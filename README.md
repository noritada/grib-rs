# grib-rs

[![License (Apache 2.0)](https://img.shields.io/badge/license-Apache%202.0-blue)](https://github.com/noritada/grib-rs/blob/master/LICENSE-APACHE)
[![License (MIT)](https://img.shields.io/badge/license-MIT-blue)](https://github.com/noritada/grib-rs/blob/master/LICENSE-MIT)
[![Build](https://github.com/noritada/grib-rs/workflows/CI/badge.svg)](https://github.com/noritada/grib-rs/actions?query=workflow%3ACI)

GRIB format parser for Rust

## About

This is a GRIB format parser library written in [Rust](https://www.rust-lang.org/) programming language. This project aims to provide a set of library and tools which is simple-to-use, efficient, and educational.

GRIB is a concise data format commonly used in meteorology to store historical and forecast weather data. It is intended to be a container of a collection of records of 2D data. GRIB files are huge and binary and should be processed efficiently. Also, since GRIB is designed to support various grid types and data compression using parameters defined in external code tables and templates, some popular existing softwares cannot handle some GRIB data.

Rust is a modern programming language that offers the performance of C and C++ but with safeguards. It has great documentation and user-friendly toolchains including a compiler with useful error messages and a package manager integrated with a build tool. Rust has been the "most loved programming language" in the [Stack Overflow Developer Survey](https://insights.stackoverflow.com/survey) every year since 2016.

## Vision

A world where everyone can read weather data easily although its interpretation needs some specific knowledge and experience.

## Features

* Rust library `grib`
  * Read and basic format checks
  * Supports of some code tables
* CLI application `gribber` built on the top of the Rust library
  * Display of some information of GRIB2 files
  * Data export as flat binary files

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

## Contributing

Contribution is always welcome.  Please check [CONTRIBUTING.md](CONTRIBUTING.md) if you are interested.

## License

This project is licensed under either of

 * Apache License, Version 2.0 (See [LICENSE-APACHE](LICENSE-APACHE)
   or http://www.apache.org/licenses/LICENSE-2.0), and
 * MIT license (See [LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
