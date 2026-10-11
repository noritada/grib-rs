# CLI

- Follow the `cli()` / `exec()` command structure in `src/commands/` and dispatch in `src/commands.rs`.
- Update README usage when help changes; `tests/cli/main.rs` compares it with actual help output. Use the existing `assert_cmd`, `predicates`, and fixture helpers in `tests/cli/utils.rs`.
- Check exit status, stdout, and stderr. For decode changes, cover affected text/binary modes, byte order, missing-value representation, and stdin/stdout behavior.
- Preserve Unix-only PROJ and pager dependencies and their platform gates in `Cargo.toml` and source.
