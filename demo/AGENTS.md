# Web demo

- Preserve WebAssembly-compatible dependencies: the demo uses pure Rust decoding backends with default library features disabled.
- Check compilation with `cargo build -p grib-demo --target wasm32-unknown-unknown`. For the Pages artifact, follow the Trunk command, working directory, and public URL in `../.github/workflows/pages.yml`.
- For interaction changes, check file drop, submessage selection, modal behavior, and canvas rendering in a browser.
