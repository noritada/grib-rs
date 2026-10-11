# Template helpers

- Preserve byte order, GRIB signed-integer representation, and cursor advancement when changing slice or buffer operations.
- When changing shared traits, update generated implementations in `../grib-template-derive/` and run both `cargo test -p grib-template-helpers` and `cargo test -p grib-template-derive`.
