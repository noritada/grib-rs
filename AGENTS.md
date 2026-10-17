# AGENTS.md

## General guidelines for development work

- To conserve bandwidth in the development environment, avoid downloading large data files unless absolutely necessary; if a download is unavoidable, obtain the user’s confirmation first. For the purposes of this guideline, "large data" refers to data files exceeding 10 MB.

## Coding style & conventions

- Do not use dynamic dispatch in library code. However, it is acceptable to use it for error handling in code examples and tests.
  - When setting the return type to `Result<(), Box<dyn std::error::Error>>` in code examples or test code, do not perform unnecessary error type conversions using `map_err`; instead, return the error directly using the `?` operator.
- For user-defined declarative macros that accept multiple arguments, as a general rule, the trailing comma in the argument list should be optional, allowing the macro to handle both cases where a trailing comma is present and where it is not, as described below. This also applies to inner levels when arguments are nested.　However, if it can be assumed with certainty that the actual arguments will be short and do not require a trailing comma, an implementation that does not expect a trailing comma is acceptable.
- When calling a user-defined declaration macro, do not include a trailing comma if there is no line break between the closing parenthesis and the element immediately preceding it; include a trailing comma if there is a line break.
- To format the code, use the nightly toolchain by running `cargo +nightly fmt`. This project uses `rustfmt.toml` to configure several formatting settings, but some of them will not work unless you use the nightly toolchain.
- For configuration files, do not include entries for optional items or those that have not been changed from their default values unless there is a specific reason to do so. When describing settings, it is most important to make it easy to recognize differences from the defaults.

## Testing instructions

- If an error unrelated to what you are testing occurs within the test code, instead of using a panic triggered by methods such as `unwrap` or `expect`, set the return type of the test function to `Result<(), Box<dyn std::error::Error>>`.
- When performing parameterized testing, you generally do not test multiple cases within a single test. However, if the subject under test is simple, it is acceptable to test multiple cases within a single test function using test sequences or loops to reduce the number of small test functions.
- When performing parameterized tests, use a macro like [`test_operation_with_data_without_nan_values`](src/decoder/tests.rs) to group a series of tests into a single block.

## Guidelines for documentation work

- When writing descriptions, make sure to use the same terminology for the same concepts and different terminology for different concepts. If various expressions are used for the same concept, readers may become confused by the differences. For example, security advisories [GHSA-6j96-629m-89w4](docs/security-advisories/GHSA-6j96-629m-89w4.md), [GHSA-m5w8-9hc6-j8x8](docs/security-advisories/GHSA-m5w8-9hc6-j8x8.md), and [GHSA-v8g4-j727-859g](docs/security-advisories/GHSA-v8g4-j727-859g.md) all use the same explanation regarding the panic issue.
- While natural language text offers a high degree of flexibility and allows for significant changes in expression, when modifying existing text within this project, limit changes—whenever possible—to simply replacing (sequences of) words in the original text to minimize differences at the word level. Since natural language text is typically longer per line than source code, line-by-line diffs tend to be too large; however, if the changes consist solely of replacing sequences of words, the changes can be effectively verified by viewing the diff at the word level.
