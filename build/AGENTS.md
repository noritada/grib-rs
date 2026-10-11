# Build-time generation

- Preserve the missing-submodule diagnostic and `cargo:rerun-if-changed` declarations in `main.rs`; register any new generation inputs.
- `doc.rs` extracts named sections from `../README.md` and features from `../Cargo.toml`. Keep those inputs consistent with the extraction code.
- Validate generation with `cargo build -p grib`. For generated-documentation changes, also run `cargo doc -p grib --no-deps` and inspect the result.
