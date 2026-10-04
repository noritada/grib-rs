# AGENTS.md

## Coding style & conventions

- Do not use dynamic dispatch in library code. However, it is acceptable to use it for error handling in code examples and tests.
  - When setting the return type to `Result<(), Box<dyn std::error::Error>>` in code examples or test code, do not perform unnecessary error type conversions using `map_err`; instead, return the error directly using the `?` operator.

## Testing instructions

- If an error unrelated to what you are testing occurs within the test code, instead of using a panic triggered by methods such as `unwrap` or `expect`, set the return type of the test function to `Result<(), Box<dyn std::error::Error>>`.
- When performing parameterized testing, you generally do not test multiple cases within a single test. However, if the subject under test is simple, it is acceptable to test multiple cases within a single test function using test sequences or loops to reduce the number of small test functions.
- When performing parameterized tests, use a macro like the one below to group a series of tests into a single block:
  ```rust
  macro_rules! parameterized_tests {
      (
          $(
              $(#[$meta:meta])*
              ($name:ident, $buf:expr, $ty:ident, $expected:expr $(,)?),
          )*
          $(,)?
      ) => ($(
          $(#[$meta])*
          #[test]
          fn $name() {
              let buf: [u8; _] = $buf;
              let actual = $ty::try_from_slice(&buf, &mut 0);
              assert_eq!(actual, $expected);
          }
      )*);
  }

  parameterized_tests! {
      (
          reading_u16,
          [0xff, 0xcc, 0xff, 0xcc],
          u16,
          Ok(0xffcc),
      ),
      (
          reading_i16,
          [0xff, 0xcc, 0xff, 0xcc],
          i16,
          Ok(-0x7fcc),
      ),
  }
  ```
