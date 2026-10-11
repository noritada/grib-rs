# Code-table build support

- Extend the CSV fixtures in `testdata/` and inline tests for affected ranges, reserved values, subtitles, and generated output.
- Run `cargo test -p grib-build`; when generated Rust changes, also build the consuming library with `cargo build -p grib`.
