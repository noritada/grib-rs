# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2020-06-07
### Added

- Initial release
- Library `grib`
  - Ability to read and check the basic structure of GRIB2
  - Decoding feature supporting Templates 5.0 (simple packing) and
    5.200 (run-length encoding packing)
- CLI application `gribber` built on the top of the Rust library
  - 4 subcommends:
    - decode: data export as text and flat binary files
    - info: display of identification information
    - inspect: display of information mainly for development purpose such as template numbers
    - list: display of a list of sections (the style is still tentative)

[0.1.0]: https://github.com/noritada/grib-rs/releases/tag/v0.1.0
