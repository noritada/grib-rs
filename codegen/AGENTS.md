# Parameter-code macro

- Preserve table path resolution relative to `CARGO_MANIFEST_DIR` and diagnostics for invalid macro input.
- `autotests` is disabled. Register new `trybuild` cases in `tests/progress.rs`, using `tests/data/table` for table fixtures.
- Review diagnostic changes before updating the corresponding `.stderr` expectations.
