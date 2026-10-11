# ecCodes interoperability helpers

- This crate wraps ecCodes through `eccodes-sys`; preserve FFI handle ownership, resource cleanup, and native error propagation.
- It is a separate workspace, outside parent `cargo test --workspace` coverage. Use `cargo test --manifest-path compat-test/Cargo.toml`.
- Before testing, check the local `grib` version requirement against the parent manifest and ensure native ecCodes dependencies are available.
