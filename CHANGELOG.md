# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2021-05-04
### Added

- Library `grib`
  - New interfaces of surfaces (submessages).
  - New interfaces of code tables.
  - Support for GRIB2 Code Tables 0.0, 3.1, 4.0 to 4.4, and 5.0.
  - New example scripts (#1):
    - examples/decode_surface.rs
    - examples/list_surfaces.rs

### Changed

- Library `grib`
  - All previous interfaces of code tables are now replaced by above-mentioned new ones.
    Old ones were just constant arrays and their features were limited.
  - Old submessage API is now replaced by above-mentioned new one.
    Old one only returned indices of relevant sections and was not useful.
  - Replace hard-coded GRIB2 Code Tables 1.1 to 1.4 with the latest ones from WMO's repository.

- CLI application `gribber` built on the top of the Rust library
  - "inspect" now shows sections and templates used in each surface (submessage).
  - "list" now shows attributes of each surface, such as an element name and forecast time.

## [0.2.0] - 2021-01-24
### Added

- Library `grib`
  - Support for Common Code Table C-11.
  - Support for Template 5.3/7.3 (complex packing).
  - Support for non-16-bits values in the Template 5.0/7.0 (simple packing) decoder
  - Support for non-8-bits values in the Template 5.200/7.200 (run-length encoding) decoder

### Changed

- Library `grib`
  - Replace Code Table 1.0 with Common Code Table C-0.

- CLI application `gribber` built on the top of the Rust library
  - Try to colorize even when using a pager.

## [0.1.0] - 2020-06-07
### Added

- Initial release
- Library `grib`
  - Ability to read and check the basic structure of GRIB2
  - Decoding feature supporting Templates 5.0/7.0 (simple packing) and
    5.200/7.200 (run-length encoding)
- CLI application `gribber` built on the top of the Rust library
  - 4 subcommends:
    - decode: data export as text and flat binary files
    - info: display of identification information
    - inspect: display of information mainly for development purpose such as template numbers
    - list: display of a list of sections (the style is still tentative)

[0.3.0]: https://github.com/noritada/grib-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/noritada/grib-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/noritada/grib-rs/releases/tag/v0.1.0
