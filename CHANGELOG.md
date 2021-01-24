# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2020-06-07
### Added

- Initial release
- Library `grib`
  - Read and basic format checks
  - Decode feature supporting Templates 5.0 and 5.200
- CLI application `gribber` built on the top of the Rust library
  - Display of some information of GRIB2 files

- CLI application `gribber` with following 4 subcommands:
  - decode: data export as text and flat binary files
  - info
  - inspect: display of information mainly for development purpose
  - list

[0.1.0]: https://github.com/noritada/grib-rs/releases/tag/v0.1.0
