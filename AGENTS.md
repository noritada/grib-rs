# AGENTS.md

Guidance for agents working on grib-rs. Direct user instructions take precedence.
Directory-level `AGENTS.md` files add local guidance; the root coding style and testing instructions still apply. Unless stated otherwise, run commands from the repository root.

## Start here

- Read [CONTRIBUTING.md](CONTRIBUTING.md) for contribution rules and [README.md](README.md) for supported templates, features, and usage.
- Check Git status and existing implementations before editing. Preserve unrelated local changes and keep the requested scope.
- Use one agent for routine work. Delegate only substantial independent tasks.
- Make repository edits in a suitable worktree under `.worktrees/`.
- Do not push, merge branches, or create or update pull requests without explicit user authorization.
- For template verification against specifications and tests, use [verify-grib-template](.agents/skills/verify-grib-template/SKILL.md).
- For numeric or representation changes affecting calculations, APIs, binary formats, and tests, use [verify-semantic-change](.agents/skills/verify-semantic-change/SKILL.md).

## Repository map

- [src/](src/AGENTS.md): the `grib` library, including parsing, decoding, encoding, grids, projections, and GRIB2 definitions.
- [cli/](cli/AGENTS.md): the `grib-cli` package and its `gribber` command.
- [demo/](demo/AGENTS.md): the Yew web demo.
- [grib-template-derive/](grib-template-derive/AGENTS.md) and [grib-template-helpers/](grib-template-helpers/AGENTS.md): template derive macros and their supporting traits.
- [gen/](gen/AGENTS.md) and [build/](build/AGENTS.md): code-table parsing and build-time generation. Generated files are written to Cargo's `OUT_DIR`; edit their sources instead.
- [codegen/](codegen/AGENTS.md): the parameter-code procedural macro.
- [def/](def/AGENTS.md): WMO table submodules. [testdata/](testdata/AGENTS.md) holds fixtures and reference outputs.
- [tests/](tests/) and [examples/](examples/AGENTS.md): integration-side test code and API usage examples.
- [compat-test/](compat-test/AGENTS.md): a separate workspace containing ecCodes interoperability helpers.
- [docs/](docs/AGENTS.md) and [.github/](.github/AGENTS.md): development documentation, advisories, workflows, and issue templates.

## Setup and checks

Initialize the submodules before building, including in a new worktree:

```sh
git submodule update --init --recursive
```

For Rust changes, run the relevant package tests during development and these checks before committing:

```sh
cargo +nightly fmt --all -- --check
cargo +nightly clippy --workspace -- -D warnings
cargo test --workspace
```

- Feature and target coverage is defined in [CI](.github/workflows/ci.yml). Run the applicable extra checks when changing optional backends, projections, or WebAssembly support.
- Inspect `Cargo.toml` for feature names. For library-only checks, use `-p grib`; workspace packages can enable library features through their own dependencies.
- Documentation-only changes need checks of links, commands, and the diff rather than a Rust build.
- Report the checks actually run and any failures or skipped checks. Do not claim CI or platform coverage from local checks alone.

## Implementation and tests

- Reuse existing helpers and patterns. Check callers, feature gates, tests, and examples when changing shared behavior or public APIs.
- Treat GRIB data as untrusted input. Preserve bounds checks, error handling, missing-value semantics, and grid ordering.
- Add focused regression coverage for behavior changes. Reuse fixtures and numerical reference outputs where applicable.
- Follow the dependency version policy in `CONTRIBUTING.md`. Add dependencies only when existing code and the standard library do not adequately solve the task.

## Coding style & conventions

- Do not use dynamic dispatch in library code. It is acceptable for error handling in examples and tests.
- In examples and tests returning `Result<(), Box<dyn std::error::Error>>`, propagate errors with `?` rather than unnecessary `map_err` conversions.
- Format with `cargo +nightly fmt --all`; some settings in `rustfmt.toml` require nightly.

## Testing instructions

- For test errors unrelated to the behavior under test, return `Result<(), Box<dyn std::error::Error>>` and use `?` instead of panicking with `unwrap` or `expect`.
- Generally give each parameterized case its own test, grouping cases with a macro as in [`test_decoder_constructor_input`](src/decoder/tests.rs). For simple subjects, a sequence or loop within one test is acceptable.
