# grib-rs

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
* CLI application `rsgrib` built on the top of the Rust library
  * Display of some information of GRIB2 files

## Planned features

* Data export as flat binary files
* WebAssembly application
* Efficient read from cloud sources such as S3
* More supports of code tables and templates
* Data extraction based on geographical specification
* Format conversion to other popular formats
* Providing interface to other languages

## CLI application `rsgrib`

```
USAGE:
    rsgrib [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help         Prints this message or the help of the given subcommand(s)
    info         Shows identification information
    list         Lists contained data
    templates    Lists used templates
```
