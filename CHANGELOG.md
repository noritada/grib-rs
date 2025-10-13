# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.13.2] - 2025-10-13
### New supports

- Reading, accessing, and dumping section/template parameters:
  - Section 1
    (#140, PR #145)
  - Templates 1.0, 1.1, 1.2
    (#140, PR #145)

### Enhancements

- Following new methods are now available:
  - `SubMessage::section1()` to access the parameters in Section 1 (and its template) of the submessage
    (#140, PR #145)

### Documentation improvements

- Documentation of parameter definition modules introduced at v0.13.1.
- Version information is now included in the CHANGELOG.md for all crates within the workspace.

### Versions

```
grib 0.13.2
grib-cli 0.13.2
grib-template-derive 0.1.1
grib-template-helpers 0.1.1
```

## [0.13.1] - 2025-10-07
### New supports

- Reading, accessing, and dumping section/template parameters:
  - Section 5
    (#140, PR #142, PR #144)
  - Templates 5.0, 5.1, 5.2, 5.3, 5.4, 5.40, 5.41, 5.42, 5.50, 5.51, 5.53, 5.61, 5.200
    (#140, PR #142, PR #144)

### Enhancements

- Following new methods are now available:
  - `SubMessage::section5()` to access the parameters in Section 5 (and its template) of the submessage
    (#140, PR #141, PR #142)
  - `SubMessage::dump()` to dump the parameters in Section 5 (and its template) of the submessage
    (#140, PR #141, PR #142)
  - `Grib2SubmessageDecoder::section5()` to access the parameters in Section 5 (and its template) of the submessage
    (#140, PR #141, PR #142)

### Enhancements for CLI application `gribber`

- New subcommand "dump" to dump the content of a GRIB submessage is now available.
  (#140, PR #143)

### Versions

```
grib 0.13.1
grib-cli 0.13.1
grib-template-derive 0.1.0
grib-template-helpers 0.1.0
```

## [0.13.0] - 2025-09-26
### Enhancements

- The separation of optional dependencies into crate features is now complete.
  Please enable or disable features as needed.
  - Decoding implementations requiring other crates have been split into separate crate features,
    allowing users who don't need them to disable them at build time.
    (#132, #134, #135)
  - As the number of crate features has increased, the `default` feature has been configured.
    (#137)
- Decoding performance improved for JPEG 2000 code stream format data.
  This improvement is currently available only when an experimental feature called `jpeg2000-unpack-with-openjpeg-experimental` is enabled.
  (#93, #136)

### Others

- Updated to the Rust 2024 Edition.
  (#139)

### Versions

```
grib 0.13.0
grib-build 0.4.4
grib-cli 0.13.0
```

## [0.12.1] - 2025-09-21
### Documentation improvements

- Crate-level documentation.
  (#138)

### Versions

```
grib 0.12.1
grib-cli 0.12.1
```

## [0.12.0] - 2025-09-08
### Enhancements

- `Grib2SubmessageDecoder::new()` is now available for decoding from byte sequences.
  (#129, #131)

### Breaking changes

- Error types for the decoder is much simplified.
  (#130)

### Others

- Updated dependencies on `png` and `proj` crates.

### Versions

```
grib 0.12.0
grib-cli 0.12.0
```

## [0.11.2] - 2025-08-05
### New supports

- Templates
  - Template 5.42/7.42 (CCSDS recommended lossless compression)
    (#94; #125)

### Others

- Fixed API documentation build failure on docs.rs
  (#124)

### Versions

```
grib 0.11.2
grib-cli 0.11.2
```

## [0.11.1] - 2025-07-14
### Documentation improvements

- Descriptions on features are now included in Cargo.toml and linked from README.md.
  (#121)
- API documentation on docs.rs and GitHub Pages now includes descriptions on feature-specific APIs.
  (#122)

### Fixes

- Correct the dependency on chrono to fix a build failure
  (#123)

### Versions

```
grib 0.11.1
grib-cli 0.11.1
```

## [0.11.0] - 2025-07-12
### New supports

- Code Tables
  - Code Table 1.2 (significance of reference time)
    (#119)

### Enhancements

- Now `IntoIter` trait is implemented for `&Grib2`.
  (#107)
- `Grib2` struct can now be created from an owned sequence of bytes as well using `from_bytes()`,
  which replaces `from_slice()` that only takes a borrowed sequence.
  (#117)
- 3 enhancements have been made to the time-related information APIs.
  (#119)
  - Introduction of the "time-calculation" feature and reduction of dependency on chrono crate.
    This crate no longer depends on chrono unless the "time-calculation" feature is explicitly enabled.
    This reduces the number of dependencies needed to build the code in cases where time calculations are not required.
  - Accessors to time-related information of a submessage.
    `SubMessage::temporal_raw_info()` and `SubMessage::temporal_info()`,
    which return reference time, forecast time, etc. in batches, are now available.
    The latter is available only when the "time-calculation" feature is enabled.
  - New feature to calculate forecast time.
    The new method `SubMessage::temporal_info()` mentioned above can be used to get the calculated forecast time.
    This forecast time is calculated from the reference time, the time elapsed from it and its units.
- An accessor, `SubMessage::identification()`, is newly introduced to return `Identification`.
  (#110 (thanks @ejd); #120)

### Enhancements for CLI application `gribber`

- CLI now uses `std::sync::LazyLock` and requires Rust 1.80 or higher for builds.
  Instead, it no longer depends on `once_cell` crate.
- Now that CLI can handle standard input/output, data can be exchanged with external commands through pipes.
  (#118)

### Others

- Suppressed warnings about untested JPEG 2000 code stream format decoding since it has passed plenty of operational testing.
  (#111 (thanks @agasparovic); #113 (thanks @jodavaho))
- As usual, followed the latest lint warnings.
  (e.g. #115 (thanks @ejd))
- As usual, updated dependencies.
  (e.g. #116 (thanks @ejd))

### Versions

```
grib 0.11.0
grib-cli 0.11.0
```

## [0.10.2] - 2024-10-02
### Fixed

- Library `grib`
  - Fixed longitude computation failures in the regular lat/lon and Gaussian grids when start/end latitudes are inconsistent with scanning mode. (#51, #103, #104)

### Versions

```
grib 0.10.2
grib-cli 0.10.2
```

## [0.10.1] - 2024-08-12
### Changed

- Library `grib`
  - Complex packing decoder now strictly checks that original field values are floating-points. (#95)

### Fixed

- Library `grib`
  - Fixed a possible issue that decoders returned wrong values when nbit is 0 and D is not 0, although no such data have been found so far. (#96)

### Versions

```
grib 0.10.1
grib-cli 0.10.1
```

## [0.10.0] - 2024-07-04
### Added

- Library `grib`
  - Support for regular Gaussian grids (Template 3.40). (#85, #90)
  - New method `SubMessage::grid_shape()` to access the grid shape without iteration. (#80)
  - New method `FixedSurface::unit()` to access the unit string defined for the type of the surface, if any. (#81)
  - New method `GridDefinitionTemplateValues::short_name()` to return the short name defined for the grid, based on ecCodes `gridType` strings. (#87)
  - New utility function `grib::utils::compute_gaussian_latitudes()` to compute Gaussian latitudes. (#92)
- Others
  - GRIB2 viewer web app for demo using the crate is now available. (#79, #83, #84)
  - Now that example code in README.md has been subject to testing, we can know any update omissions. (#89)

### Changed

- Library `grib`
  - `SubMessage::latlons()` now strictly returns an error for quasi-regular latitude/longitude grids as unsupported. (#86)
  - `LatLonGridIterator` has been renamed to `RegularGridIterator`. (#88)

### Contributors

- Thanks for sharing the data that cannot be processed.
  - @BruAPAHE (#85)

### Versions

```
grib 0.10.0
grib-cli 0.10.0
```

## [0.9.2] - 2024-05-24
### Added

- Library `grib`
  - Support for first-order spatial differencing in the complex packing decoder. (#78)
  - Support for Code Table 5.6 (order of spatial differencing). (#78)

### Contributors

- Thanks for sharing the data that cannot be processed.
  - @animus27 (#77)

### Versions

```
grib 0.9.2
grib-cli 0.9.2
```

## [0.9.1] - 2024-04-03
### Fixed

- Library `grib`
  - Fix `FixedValueIterator` crash after iteration completion. (#75, #76)

### Contributors

- Thanks for reporting a bug and providing its fix.
  - @shastro (#75, #76)

### Versions

```
grib 0.9.1
grib-build 0.4.3
grib-cli 0.9.1
```

## [0.9.0] - 2024-02-04

### Added

- Library `grib`
  - New feature `gridpoints-proj` to compute coordinates of grid points using `proj` crate. (#69)
  - Support for Code Table 3.2 (shape of the reference system). (#70)
  - Support for Code Table 3.5 (projection centre). (#73)
  - Support for Template 3.20 (polar stereographic). (#73)
    - Feature `gridpoints-proj` needs to be enabled to use this support.
  - Support for Template 3.30 (Lambert conformal). (#69, #71, #72)
    - Feature `gridpoints-proj` needs to be enabled to use this support.
  - Grid point index iterator API to allow users to use grid point indices. (#67)
  - Support for building WASM. (#65)

### Changed

- Library `grib`
  - Improved the module structure to make the source code a little more readable and easier to contribute. (#66)
- Others
  - Improved descriptions on template support in README. (#74)

### Versions

```
grib 0.9.0
grib-cli 0.9.0
```

## [0.8.0] - 2023-11-11
### Added

- Library `grib`
  - Support for GRIB files with non-GRIB byte sequences between messages. (#60)
  - Support for Code Table 5.5 (missing value management for complex packing). (#61)
  - Support for Template 5.2/7.2 (PNG format). (#62)
  - New example scripts:
    - examples/check_decoding_of_files.rs
- Others
  - The project has begun using GitHub Discussions as a forum. (#58)

### Changed

- Library `grib`
  - Due to support for the input with non-GRIB byte sequences between messages, the reader now behaves differently (#60):
    - If non-GRIB byte sequences are included in the input, the reader used to return an error, but now it skips those sequences and try to find the next GRIB message in the input.
    - If there is no GRIB data in the input byte sequence, the reader used to return an error, but now it successfully reads empty data.
  - Non-API changes:
    - Users are now suggested to run `git submodule update --init` in build failure. (#57)

### Fixed

- Library `grib`
  - Fixed an issue that the decoder returned wrong values when nbit is 0 in Template 5.3/7.3 (complex packing and spatial differencing). (#63)

### Contributors

- Thanks for letting us know about the need for the forum.
  - Tom Clark
- Thanks for sharing the data that cannot be processed.
  - @sapiennervosa (#59)

### Versions

```
grib 0.8.0
grib-cli 0.8.0
```

## [0.7.1] - 2023-04-20
### Added

- Library `grib`
  - Support for Template 5.41/7.41 (PNG format). (#53, #55)
- Others
  - GRIB tools from ecCodes are now mentioned in a section on comparison of GRIB processors in the cookbook. (#52)

### Fixed

- Library `grib`
  - Fixed an issue that `size_hint()` results do not change after consuming iterator items. (#50, #54)

### Versions

```
grib 0.7.1
grib-cli 0.7.1
```

## [0.7.0] - 2023-02-09
### Added

- Library `grib`
  - Support for computation of latitudes and longitudes of grid points (for the lat/lon grid (Template 3.0)). (#39, #41, #43)

### Changed

- Library `grib`
  - Now most module hierarchies in the crate have been removed from API and all types except code tables are available directly under the crate root.
    This makes it easier to import types and get a full picture of the types in the crate via API references.
    Note that type names are not changed in this release. (#44, #47)
  - Non-API changes:
    - Cleaned up superfluous use statements. (#45)
- CLI application `gribber` built on the top of the Rust library
  - The "decode" subcommand now prints grid points' lat/lon values as well as data. (#40)
- Others
  - Now README.md includes example code to show the usage of the API. (#48)
  - The word "surface" used in README.md and elsewhere to describe a submessage is now replaced with "layers". (#49)

### Fixed

- Library `grib`
  - Fixed an issue that wrong offset values are stored in reading a message starting from a non-zero position, which may result in incorrect value output, panics, etc. (#38, #42)
  - Fixed an issue that wrong Section 0 and 1 information is linked with submessages returned from iterators for multi-message data. (#37, #46)

### Contributors

- Thanks for reporting issues fixed in this release:
  - @LafeWessel (#37, #38)

### Versions

```
grib 0.7.0
grib-build 0.4.2
grib-cli 0.7.0
```

## [0.6.1] - 2023-01-29
### Added

- Library `grib`
  - Support for zero width groups in Template 5.3/7.3 decoder. (#31)
  - Support for the number of spatial differencing extra descriptors other than 2 in Template 5.3/7.3 decoder. (#33, #36)
  - Support for cases where nbit is 0 in following decoders (#35):
    - Template 5.3/7.3 (complex packing and spatial differencing)
    - Template 5.40/7.40 (JPEG 2000 code stream format)
- CLI application `gribber` built on the top of the Rust library
  - New subcommand "completions" to generate shell completions.

### Changed

- Library `grib`
  - Verify that encoding parameter values are supported before decoding the data encoded with complex packing.
- CLI application `gribber` built on the top of the Rust library
  - The version of `clap` used is now 4.1 (no change in behavior except for messages).

### Fixed

- Library `grib`
  - Fixed a panic with an assertion failure (with a message "assertion failed" in Template 5.3/7.3 decoder. (#29, #31)
    - Correctly use spatial differencing extra descriptors in Template 7.3.
    - Correct offset bit calculation in handling non-zero width groups.
  - Fixed a panic with an index out-of-bounds error (with a message "range end index 4 out of range for slice of length 3") in Template 5.3/7.3 decoder. (#32, #35)

### Contributors

- Thanks for reporting issues concerning this release:
  - @LafeWessel (#29, #32)

### Versions

```
grib 0.6.1
grib-build 0.4.1
grib-cli 0.6.1
```

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

### Versions

```
grib 0.6.0
grib-build 0.4.0
grib-cli 0.6.0
```

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

### Versions

```
grib 0.5.0
```

## [0.4.3] - 2022-01-15

### Changed

- CLI application `gribber` built on the top of the Rust library
  - The performance of the "decode" subcommand is improved for big-endian binary output.

### Versions

```
grib 0.4.3
```

## [0.4.2] - 2022-01-13

### Changed

- CLI application `gribber` built on the top of the Rust library
  - The performance of the "decode" subcommand is improved.
  - The version of `clap` used is now 3 (no change in behavior except for messages).

### Versions

```
grib 0.4.2
```

## [0.4.1] - 2022-01-03
### Added

- Library `grib`
  - Simple and easy-to-use reading APIs `from_reader())` and `from_slice()`.

### Versions

```
grib 0.4.1
```

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

### Versions

```
grib 0.4.0
grib-build 0.3.0
```

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

### Versions

```
grib 0.3.0
grib-build 0.2.0
```

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

### Versions

```
grib 0.2.0
grib-build 0.1.0
```

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

### Versions

```
grib 0.1.0
```

[unreleased]: https://github.com/noritada/grib-rs/compare/v0.13.2...HEAD
[0.13.2]: https://github.com/noritada/grib-rs/compare/v0.13.1...v0.13.2
[0.13.1]: https://github.com/noritada/grib-rs/compare/v0.13.0...v0.13.1
[0.13.0]: https://github.com/noritada/grib-rs/compare/v0.12.1...v0.13.0
[0.12.1]: https://github.com/noritada/grib-rs/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/noritada/grib-rs/compare/v0.11.2...v0.12.0
[0.11.2]: https://github.com/noritada/grib-rs/compare/v0.11.1...v0.11.2
[0.11.1]: https://github.com/noritada/grib-rs/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/noritada/grib-rs/compare/v0.10.2...v0.11.0
[0.10.2]: https://github.com/noritada/grib-rs/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/noritada/grib-rs/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/noritada/grib-rs/compare/v0.9.2...v0.10.0
[0.9.2]: https://github.com/noritada/grib-rs/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/noritada/grib-rs/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/noritada/grib-rs/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/noritada/grib-rs/compare/v0.7.1...v0.8.0
[0.7.1]: https://github.com/noritada/grib-rs/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/noritada/grib-rs/compare/v0.6.1...v0.7.0
[0.6.1]: https://github.com/noritada/grib-rs/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/noritada/grib-rs/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/noritada/grib-rs/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/noritada/grib-rs/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/noritada/grib-rs/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/noritada/grib-rs/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/noritada/grib-rs/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/noritada/grib-rs/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/noritada/grib-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/noritada/grib-rs/releases/tag/v0.1.0
