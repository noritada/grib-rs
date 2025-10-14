pub(crate) mod common;
pub(crate) mod decode;
pub(crate) mod dump;
pub(crate) mod info;
pub(crate) mod inspect;
pub(crate) mod list;

macro_rules! test_simple_display {
    ($(($name:ident, $command:expr, $input:expr, $options:expr, $expected_stdout:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;
            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($command).args($options).arg(input.path());
            cmd.assert()
                .success()
                .stdout(predicate::str::diff($expected_stdout))
                .stderr(predicate::str::is_empty());

            Ok(())
        }
    )*);
}
pub(crate) use test_simple_display;
