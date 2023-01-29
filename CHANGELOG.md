# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0] - 2022-11-13
### Added

- Library `grib`
  - Multi-message data support in `Grib2` using newly introduced iterator-based parsers internally.
  - New method `Grib2::len()` for submessage list length checking.
  - New `Grib2SubmessageDecoder`, which returns an iterator of decoded values in a submessage. (#27)
  - New experimental parser API `Grib2SubmessageStream`, which parses GRIB2 data as an iterator.
    Currently this is experimental and users cannot use decoders with this API.
- CLI application `gribber` built on the top of the Rust library
  - Multi-message data support. (#21)

### Changed

- Library `grib`
  - `Grib2` is now re-exported at the crate root.
  - Unnecessarily strong trait boundaries for `Grib2::iter()`, `Grib2::submessages()`, and `Grib2::sections()` are removed.
  - `Grib2::sections()` now return an iterator instead of a slice.
  - `Identification::ref_time()` now returns `Result` instead of directly returning `chrono::DateTime`.
    So, this method will not panic even if the reference time in the GRIB file is invalid. (#28)
  - `SubMessageIterator` is renamed as `SubmessageIterator`.
  - Non-API changes
    - Development version of the library's API documentation is now available on GitHub Pages. (#22)
- CLI application `gribber` built on the top of the Rust library
  - The application now uses `anyhow` for error handling instead of a custom error type in order to reduce boilerplate code.
  - The version of `clap` used is now 4.0. (#26)
  - User-invisible changes
    - Duplication in test code for CLI has been eliminated so that commonalities and differences between test cases get clarified.
    - Test code for CLI has been reorganized for better accessibility.
- Others
  - Source code for CLI has been separated from the "grib" library package as "grib-cli" for the separation of dependencies. (#23)
  - Cookbook has been migrated from mdBook to rustdoc so that the code in the documentation is now always tested against the library's API. (#25)
  - `aarch64-apple-darwin` is added to the target architecture list in nightly building.
  - Enabled link time optimization and symbol stripping in release builds to improve runtime performance and reduce the size of builds.

### Removed

- Library `grib`
  - `Grib2::scan()` has been removed in favor of newly introduced iterator-based parsers.
  - `Grib2::info()` has been removed since it does not make sense for multi-message data.
  - `Grib2::get_values()` has been removed in favor of newly introduced `Grib2SubmessageDecoder`. (#24)
  - An enum variant `GribError::ValidationError` has been removed since its functionality is now included in `GribError::ParseError`.

### Fixed

- CLI application `gribber` built on the top of the Rust library
  - Fixed an issue that "Something unexpected happend" is shown when a wrong message index is given to the "decode" subcommand

## [0.5.0] - 2022-07-10
### Added

- Library `grib`
  - Support for Section 6 (Bit-map Section). (#19)

### Changed

- Library `grib`
  - There is a small change in arguments of `decoders::dispatch()`.
- CLI application `gribber` built on the top of the Rust library
  - The "list" subcommand now displays the number of grid points and ones whose values are NaN. (#20)
  - The version of `clap` used is now 3.2 (no change in behavior).

### Contributors

- Thanks for sending PRs to this release:
  - @resistor (#18)

## [0.4.3] - 2022-01-15

### Changed

- CLI application `gribber` built on the top of the Rust library
  - The performance of the "decode" subcommand is improved for big-endian binary output.

## [0.4.2] - 2022-01-13

### Changed

- CLI application `gribber` built on the top of the Rust library
  - The performance of the "decode" subcommand is improved.
  - The version of `clap` used is now 3 (no change in behavior except for messages).

## [0.4.1] - 2022-01-03
### Added

- Library `grib`
  - Simple and easy-to-use reading APIs `from_reader())` and `from_slice()`.

## [0.4.0] - 2021-12-29
### Added

- Library `grib`
  - New method `Grib2::iter()` for iteration over submessages.
  - Support for GRIB2 Code Table 4.5.
  - Support for Template 5.40/7.40 (JPEG 2000 code stream format).
  - Support for cases where nbit is 0 in the Template 5.0/7.0 (simple packing) decoder.
- Others
  - Documentation on comparison of GRIB2 data operations among tools.

### Changed

- Library `grib`
  - Reorganize APIs of code tables, data types and decoders.
  - `datatypes::ForecastTime` structs are now used for the data representation of forecast time
    instead of tuples.
- Others
  - Update to the Rust 2021 Edition

### Contributors

- Thanks for sending PRs to this release:
  - @Quba1
  - @crepererum
  - @mulimoen

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
  - Support for Template 5.3/7.3 (complex packing and spatial differencing).
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
  - 4 subcommands:
    - decode: data export as text and flat binary files
    - info: display of identification information
    - inspect: display of information mainly for development purpose such as template numbers
    - list: display of a list of sections (the style is still tentative)

[unreleased]: https://github.com/noritada/grib-rs/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/noritada/grib-rs/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/noritada/grib-rs/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/noritada/grib-rs/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/noritada/grib-rs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/noritada/grib-rs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/noritada/grib-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/noritada/grib-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/noritada/grib-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/noritada/grib-rs/releases/tag/v0.1.0
