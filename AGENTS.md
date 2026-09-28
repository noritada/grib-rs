# AGENTS.md

## Coding style & conventions

- Do not use dynamic dispatch in library code. However, it is acceptable to use it for error handling in code examples and tests.

## Testing instructions

- If an error unrelated to what you are testing occurs within the test code, instead of using a panic triggered by methods such as `unwrap` or `expect`, set the return type of the test function to `Result<(), Box<dyn std::error::Error>>`.
- When performing parameterized testing, you generally do not test multiple cases within a single test. However, if the subject under test is simple, it is acceptable to test multiple cases within a single test function using test sequences or loops to reduce the number of small test functions.
