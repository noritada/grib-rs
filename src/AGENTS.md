# Library

- Reuse fixtures through `test_utils.rs` and `test_utils/data.rs`; decoder regression tests live in `decoder/tests.rs`.
- For grid and projection changes, check affected built-in and `gridpoints-proj` paths and the correspondence between coordinates and decoded values.
- Preserve backend selection: hayro takes priority over OpenJPEG, and rust-aec over libaec when both corresponding features are enabled.
