# Template derive macros

- Keep generated implementations consistent with the traits in `../grib-template-helpers/` and their callers in `../src/def/`.
- `autotests` is disabled. Register new cases in `tests/runner.rs`; adding a `.rs` file alone does not run it.
- Existing `trybuild` pass cases include executable assertions. Check generated behavior as well as compilation.
