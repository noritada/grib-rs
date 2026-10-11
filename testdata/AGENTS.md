# Test fixtures

- Read `README.md` before regenerating data. Record the source, message/submessage, reproduction command, endianness, and missing-value conventions for new reference outputs.
- Keep wgrib2 numerical references independent of the implementation under test.
- Before renaming or deleting fixtures, check consumers in `../src/test_utils/data.rs`, `../cli/tests/cli/utils/testdata.rs`, and the affected tests.
