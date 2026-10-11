---
name: verify-grib-template
description: Use when checking a GRIB2 template's binary layout, calculation rules, implementation coverage, or test evidence in grib-rs against its specification. Not for unrelated refactoring or a repository-wide security audit.
---

# Verify a GRIB template

Produce a traceable comparison of the applicable specification, executable code, and test assertions. Verification does not authorize fixes. Follow the root [AGENTS.md](../../../AGENTS.md), [CONTRIBUTING.md](../../../CONTRIBUTING.md), and instructions local to inspected files; do not duplicate or relax the maintainer's coding and testing conventions.

Repository paths below are relative to the repository root. Run commands there. Return the report in the user's language; keep GRIB names, identifiers, and source terminology precise.

## Establish the target

- Identify the edition, section and template number, and requested capability: parsing, metadata access, coordinate calculation, decoding, or encoding. A template enum variant proves none of the later capabilities by itself.
- Record the Git revision and relevant uncommitted changes. For a supplied message, identify its message/submessage index, originating centre, and master/local table versions from Sections 0 and 1.
- If only a template number is given, inspect the corresponding implementation and state the specification edition chosen. Ask only when an unresolved edition, local extension, or capability would materially change the result. This repository's definitions are GRIB2; do not silently reinterpret a GRIB1 request.

## Establish the sources

Use these entry points to locate and read the exact template, referenced tables, notes, regulations, and applicable amendments:

1. [WMO Manual on Codes, Volume I.2](https://community.wmo.int/site/knowledge-hub/programmes-and-initiatives/wmo-information-system-wis/about-manual-codes-volume-i2). Record edition, amendment status/effective date, and page, clause or note. A draft or newer amendment does not automatically govern an older message.
2. [WMO machine-readable codes](https://community.wmo.int/site/knowledge-hub/programmes-and-initiatives/wmo-information-system-wis/about-manual-codes-volume-i2/latest-version) and the pinned `def/GRIB2` and `def/CCT` submodules. Inspect `git submodule status` and `git ls-tree HEAD def/GRIB2 def/CCT`. An uninitialized submodule is unavailable evidence, not an empty specification. Record the revision actually read; do not update to upstream HEAD for a verification. The WMO Manual prevails over conflicting machine-readable entries.
3. [ECMWF's GRIB2 format reference](https://codes.ecmwf.int/grib/format/grib2/) is a useful cross-check and navigation entry point. For interoperability, consult versioned ecCodes/wgrib2 source or documented provider extensions when relevant. Their behavior does not establish a WMO requirement.

For each source, record its URL or local path, version/revision when available, and precise locator. Read the source rather than relying on search snippets or code comments. If the applicable normative text is inaccessible or contradictory, continue the code/test comparison but mark normative compliance unresolved. Do not promote an implementation convention into a specification rule.

## Trace specification to code

Start with a bounded search, for example `rg -n 'Template5_0|SimplePacking' src grib-template-helpers grib-template-derive`. Follow shared structures and helpers only as needed for the selected template.

| Concern | Repository entry points |
| --- | --- |
| Section headers, template selection, shared parameters | `src/def/grib2.rs`, `src/def/grib2/template{1,3,4,5}.rs` |
| Generated readers/writers and scalar representation | `grib-template-derive/src/`, `grib-template-helpers/src/` |
| Metadata and time calculations | `src/datatypes/`, `src/time.rs` |
| Coordinates and scanning order | `src/grid.rs`, `src/grid/`, `src/projection/` |
| Packing, unpacking, bitmap and value counts | `src/decoder.rs`, `src/decoder/`, `src/encoder/` |
| Code tables and generated data | `src/codetables/`, `gen/`, `build/`, `def/` |

For the requested capability:

- Map WMO's one-based section octets to the actual Rust buffer and cursor. Establish whether a slice includes the section header, section payload, or template; account for shared parameter structures and variable-length fields. Do not equate a template offset with a section offset.
- Compare field width, byte order, signed representation, units, scale factors, missing/reserved values, and conditional/repeated fields. Follow `TryFromSlice`, `WriteToBuffer`, and their derives where they determine layout. GRIB signed-integer conversion uses helpers such as `try_from_slice/as_grib_signed.rs`; an `i16` field alone does not explain wire representation.
- Trace dispatch and feature gates through the actual calculation. Distinguish field storage precision from intermediate arithmetic and output precision. Check equations, exponent signs, units, count/order semantics, and template-specific special cases against the cited notes.
- Derive a small expected result independently from the specification for a disputed calculation. Include a boundary that distinguishes plausible implementations, rather than only zero or identity values. Inspect executable helpers when a comment and implementation disagree.

## Check the evidence in tests

Locate the relevant inline tests, `src/decoder/tests.rs`, derive test runners, and fixture consumers. Read the assertion and its inputs, not just the test name.

- Establish what would make each test fail. An unused predicate, compilation-only coverage, or a round trip through matching encoder/decoder mistakes does not prove numerical agreement with the specification.
- Select relevant cases: ordinary values, signed/scaled values, missing values, zero-width or constant fields, bitmap counts, scan order, optional/repeated fields, and truncated input. Apply only cases relevant to the target; distinguish format conformance from malformed-input robustness.
- Consult `testdata/README.md`, `src/test_utils/data.rs`, and, if the CLI is in scope, `cli/tests/cli/utils/testdata.rs`. For external reference output, record fixture, message index, tool/version when known, options, byte order, missing-value handling, and tolerance. Do not regenerate expectations from the implementation being checked.
- For changed template helpers or derives, include their package tests and registered `trybuild` cases. For optional codecs, projections, or time support, take feature combinations from `Cargo.toml` and `.github/workflows/ci.yml`. Workspace dependencies can enable features absent from a library-only check.

Discover the selected tests before claiming coverage. For example, for the simple-packing unit tests:

```sh
cargo test -p grib --no-default-features decoder::simple::tests -- --list
cargo test -p grib --no-default-features decoder::simple::tests
```

Adapt the filter after inspecting actual test names. These commands compile code and need the build prerequisites documented at the root, including pinned submodules. A zero-test selection is not a pass for the requested behavior. If the user requests source inspection only, propose commands without executing them; otherwise run the relevant tests and record command, feature set, test count, and result. Separate prerequisite/build failures from test failures. Follow the root checks if a separately authorized fix changes code.

## Deliver the comparison

Lead with a bounded verdict for the requested capability and list the revision, specification baseline, and configuration used. Summarize the important rules in this matrix:

| Rule or calculation | Source and locator | Code and locator | Test/assertion and execution status | Conclusion |
| --- | --- | --- | --- | --- |

Use conclusions such as **consistent by inspection**, **mismatch**, or **unresolved**. Keep test execution status explicit: passed, failed, not run, or blocked. A test gap is not automatically an implementation defect, and passing sampled cases does not establish full template conformance.

For a mismatch, give the triggering input, expected and observed behavior, source, and code location. For an unresolved rule, name the missing source or evidence. End with the smallest additional tests needed, including their expected oracle and why existing assertions do not cover them. Report in chat unless the user requests a saved artifact; preserve source code and fixtures during verification.
