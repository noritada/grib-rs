# WMO definitions

- Make parser and generator changes in `../gen/` and `../build/`; do not patch external WMO tables to conceal a parser failure.
- For an intentional submodule revision update, inspect the gitlink diff and validate the new definitions with `cargo test -p grib-build` and `cargo test -p grib`.
