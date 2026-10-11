---
name: verify-semantic-change
description: Use when assessing a proposed or implemented numeric, type, unit, or representation change in grib-rs, especially f32 to f64 migrations affecting calculations, public APIs, binary formats, and test expectations. Not for purely mechanical renames or a repository-wide audit.
---

# Verify a semantic change

Trace the intended behavior change through conversions, calculations, APIs, consumers, serialization, and assertions. Verification does not authorize implementation or fixture updates. Follow root [AGENTS.md](../../../AGENTS.md), [CONTRIBUTING.md](../../../CONTRIBUTING.md), and applicable local instructions. Repository paths below start at the root. Report in the user's language.

## Establish the comparison and contract

- Record the requested scope, Git revision, relevant staged/unstaged changes, and untracked files. For an implemented change, identify the actual baseline and target commits or working-tree diff; do not assume `origin/master` is the baseline. For a proposal without a patch, deliver an impact assessment and planned checks, not a verdict on migrated code.
- Separate the intended numerical change from invariants: units, missing values, count/order, public types, GRIB layout, CLI binary layout, exact output bytes, and text output. Preserving a format does not necessarily preserve values or bytes. Ask only when an unresolved contract materially changes the verdict or implementation.
- Follow the affected producer-to-consumer paths. Inspect callers and feature gates; do not treat every `f32` occurrence as a migration target. For a shared helper, distinguish unavoidable propagation from unrelated algorithms that merely need an output adapter.

## Follow the representation boundaries

Use bounded searches for the changed symbols, casts, conversion methods, literal suffixes, and serialization calls. These are entry points, not an exhaustive or permanently accurate inventory:

| Boundary | Inspect in this repository |
| --- | --- |
| GRIB fields and generated layout | `src/def/grib2.rs`, `src/def/grib2/template5.rs`, `grib-template-derive/src/`, `grib-template-helpers/src/` |
| Decoding arithmetic and constants | `src/decoder/simple.rs`, `src/decoder/stream.rs`, `SimplePacking::zero_bit_reference_value` |
| Public output, dispatch, missing values | `src/decoder.rs`, `src/decoder/bitmap.rs`, shared codec callers, `src/decoder/run_length.rs`, exports in `src/lib.rs` |
| Encoding and quantization | `src/encoder.rs`, `src/encoder/simple.rs`, affected packing strategies |
| Consumers and output formats | `cli/src/commands/decode.rs`, `demo/src/main.rs`, `demo/src/palette.rs`, examples and README/doctests |

For each relevant boundary, record input type/unit, intermediate type, output type/unit, conversion purpose, and expected behavior before/after.

- **Calculations:** trace the earliest precision loss, not only the return type. Check `to_f32`/`to_f64`, casts, literal inference, `powi`, rounding, cancellation, exponent signs, and operation order. Promoting an already rounded result cannot recover lost precision. Include zero-width/constant branches and missing-value paths.
- **Storage:** distinguish on-wire precision from arithmetic precision. `SimplePacking::ref_val` is an IEEE binary32 field; wider calculation does not imply a wider GRIB field. Follow readers, writers, derives, cursor movement, byte counts, signed-integer helpers, and `to_bits`/byte conversion implementations before changing their types. Use [verify-grib-template](../verify-grib-template/SKILL.md) when a GRIB rule needs specification verification.
- **APIs:** follow iterator item types, trait bounds, enum dispatch, public exports, and explicitly typed downstream callers. Changing an exposed `f32` to `f64` can break source compatibility. Check shared simple-packing users, optional codecs, and run-length adaptation separately. Encoding inputs and coordinates already use `f64` in relevant paths; verify their actual role before widening the scope.
- **Outputs:** inspect the CLI's four-byte big/little-endian output and explicit narrowing boundary. Establish overflow, rounding, missing-value sentinel, and text-format expectations. An `is_nan()` assertion proves classification, not a required NaN byte pattern. Check the demo palette and examples for consumer assumptions.

Read the [Rust numeric-cast rules](https://doc.rust-lang.org/reference/expressions/operator-expr.html#numeric-cast) and [floating-point NaN guarantees](https://doc.rust-lang.org/std/primitive.f32.html#nan-bit-patterns) for disputed language behavior. Inspect conversion trait implementations too; they need not behave like `as`. Record versions or revisions of sources used. Do not assume all narrowing conversions reject overflow or preserve NaN payloads.

## Choose a discriminating numerical case

For simple packing, `(R + X * 2^E) * 10^-D`, one useful case is:

```text
R = -16_777_216 (exact binary32), X = 16_777_217 (25-bit code), E = D = 0
f32 arithmetic: 0, because X first rounds to 16_777_216
f64 arithmetic with promotion before calculation: 1
```

Promoting the old result to `f64` still gives zero. Narrowing the new result for a four-byte CLI output gives one, so the layout stays the same while the bytes change. This arithmetic example is an oracle candidate, not evidence that the repository's migrated decoder was executed. Treat the difference according to the agreed contract; do not automatically label it a regression or overwrite the reference output.

## Check assertions and run the relevant tests

- Inspect actual assertions in inline decoder tests, `src/decoder/tests.rs`, encoder round trips, and `cli/tests/cli/commands/decode.rs`. A discarded `.all(...)` result checks nothing; follow its use before judging it. A round trip alone can hide matching encoder/decoder errors.
- Obtain independent expected values from a small exact calculation or a documented external reference. Record fixture, tool/version, options, byte order, and missing-value handling from `testdata/README.md` and fixture consumers. Existing four-byte reference files must still be read as `f32` before any promotion; they cannot demonstrate additional precision by themselves.
- Select cases relevant to the change: cancellation above; nonzero binary/decimal scales; zero-bit fields with nonzero reference and scale; bitmap and complex missing values; rounding and accepted overflow/underflow boundaries; signed zero/NaN when observable. Preserve counts, order, and iterator contracts. For narrowing, include values near the destination's representable limits.
- Choose equality deliberately: exact bytes for a byte contract, exact values for exact cases, justified absolute/relative or ULP bounds for approximations. Explain the oracle and tolerance. Do not replace expected values with target-generated output or loosen tolerances merely to pass.
- Discover selected tests with `-- --list` before executing the inspected filters. Compare baseline and target using the same fixtures, toolchain, and features when a patch exists. A zero-test selection is not coverage. Distinguish pre-existing failures from introduced ones.
- Follow root setup/checks and CI feature combinations. Include affected API doctests/examples, CLI binary/text assertions, shared backends, and demo/Wasm compilation when their paths change. Workspace feature unification can hide a library-only configuration; `compat-test` is a separate workspace and its manifest must be inspected before claiming coverage.

For source-only or pre-implementation assessments, list proposed commands and missing cases without claiming execution. Otherwise run the relevant checks and report commands, selected tests, configuration, results, and prerequisite failures. Run the root Rust checks if a separately authorized fix modifies Rust code.

## Deliver a bounded verdict

Lead with impact assessment or patch verdict, followed by the baseline/target and intended behavior. Use a compact matrix:

| Contract or intended change | Before/after and code location | Oracle and assertion | Execution status | Conclusion |
| --- | --- | --- | --- | --- |

Distinguish intended differences, compatibility breaks, defects, and unresolved evidence. Give a triggering input for a mismatch. End with the smallest missing tests or contract decisions; separate static inspection from executed proof. Report in chat unless a saved artifact is requested. Leave unrelated defects as scoped notes.
