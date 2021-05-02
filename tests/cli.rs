use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

mod utils;

const CMD_NAME: &'static str = "gribber";

#[test]
fn help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(
            predicate::str::contains("USAGE:")
                .and(predicate::str::contains("FLAGS:"))
                .and(predicate::str::contains("SUBCOMMANDS:")),
        )
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn no_subcommand_specified() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("--help");
    let help_msg = cmd.output()?.stdout;
    let help_msg = format!("{}", String::from_utf8(help_msg)?);

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::similar(help_msg));

    Ok(())
}

#[test]
fn no_such_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("foo");
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::starts_with(
                "error: Found argument 'foo' which wasn't expected, or isn't valid in this context",
            )
            .and(predicate::str::contains("USAGE:"))
            .and(predicate::str::contains("SUBCOMMANDS:").not()),
        );

    Ok(())
}

#[test]
fn info() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "\
Discipline:                             Meteorological products
Total Length:                           10321
Originating/generating centre:          Tokyo (RSMC), Japan Meteorological Agency
Originating/generating sub-centre:      0
GRIB Master Tables Version Number:      5 (4 November 2009)
GRIB Local Tables Version Number:       1 (Number of local tables version used)
Significance of Reference Time:         Analysis
Reference time of data:                 2016-08-22 02:00:00 UTC
Production status of processed data:    Operational products
Type of processed data:                 Analysis and forecast products
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("info").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn list() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "\
0
Grid:                                   Latitude/longitude
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Analysis
  Forecast Time:                        0
  Forecast Time Unit:                   Minute
Data Representation:                    Run length packing with level values

1
Grid:                                   Latitude/longitude
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        10
  Forecast Time Unit:                   Minute
Data Representation:                    Run length packing with level values

2
Grid:                                   Latitude/longitude
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        20
  Forecast Time Unit:                   Minute
Data Representation:                    Run length packing with level values

3
Grid:                                   Latitude/longitude
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        30
  Forecast Time Unit:                   Minute
Data Representation:                    Run length packing with level values

4
Grid:                                   Latitude/longitude
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        40
  Forecast Time Unit:                   Minute
Data Representation:                    Run length packing with level values

5
Grid:                                   Latitude/longitude
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        50
  Forecast Time Unit:                   Minute
Data Representation:                    Run length packing with level values

6
Grid:                                   Latitude/longitude
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        60
  Forecast Time Unit:                   Minute
Data Representation:                    Run length packing with level values

";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("list").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn inspect() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "\
Sections:
    0 │ 0000000000000000 - 0000000000000010 │ Section 0
    1 │ 0000000000000010 - 0000000000000025 │ Section 1
    2 │ 0000000000000025 - 000000000000006d │ Section 3
    3 │ 000000000000006d - 000000000000008f │ Section 4
    4 │ 000000000000008f - 00000000000000a6 │ Section 5
    5 │ 00000000000000a6 - 00000000000000ac │ Section 6
    6 │ 00000000000000ac - 000000000000061b │ Section 7
    7 │ 000000000000061b - 000000000000063d │ Section 4
    8 │ 000000000000063d - 0000000000000654 │ Section 5
    9 │ 0000000000000654 - 000000000000065a │ Section 6
   10 │ 000000000000065a - 0000000000000bd1 │ Section 7
   11 │ 0000000000000bd1 - 0000000000000bf3 │ Section 4
   12 │ 0000000000000bf3 - 0000000000000c0a │ Section 5
   13 │ 0000000000000c0a - 0000000000000c10 │ Section 6
   14 │ 0000000000000c10 - 000000000000118c │ Section 7
   15 │ 000000000000118c - 00000000000011ae │ Section 4
   16 │ 00000000000011ae - 00000000000011c5 │ Section 5
   17 │ 00000000000011c5 - 00000000000011cb │ Section 6
   18 │ 00000000000011cb - 000000000000173e │ Section 7
   19 │ 000000000000173e - 0000000000001760 │ Section 4
   20 │ 0000000000001760 - 0000000000001777 │ Section 5
   21 │ 0000000000001777 - 000000000000177d │ Section 6
   22 │ 000000000000177d - 0000000000001cf0 │ Section 7
   23 │ 0000000000001cf0 - 0000000000001d12 │ Section 4
   24 │ 0000000000001d12 - 0000000000001d29 │ Section 5
   25 │ 0000000000001d29 - 0000000000001d2f │ Section 6
   26 │ 0000000000001d2f - 00000000000022a4 │ Section 7
   27 │ 00000000000022a4 - 00000000000022c6 │ Section 4
   28 │ 00000000000022c6 - 00000000000022dd │ Section 5
   29 │ 00000000000022dd - 00000000000022e3 │ Section 6
   30 │ 00000000000022e3 - 000000000000284d │ Section 7
   31 │ 000000000000284d - 0000000000002851 │ Section 8

SubMessages:
   id │    S2    S3    S4    S5    S6    S7 │ Tmpl3   Tmpl4   Tmpl5
    0 │     -     2     3     4     5     6 │ 3.0     4.0     5.200  
    1 │     -     2     7     8     9    10 │ 3.0     4.0     5.200  
    2 │     -     2    11    12    13    14 │ 3.0     4.0     5.200  
    3 │     -     2    15    16    17    18 │ 3.0     4.0     5.200  
    4 │     -     2    19    20    21    22 │ 3.0     4.0     5.200  
    5 │     -     2    23    24    25    26 │ 3.0     4.0     5.200  
    6 │     -     2    27    28    29    30 │ 3.0     4.0     5.200  

Templates:
3.0      - Latitude/longitude
4.0      - Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
5.200    - Run length packing with level values
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn inspect_with_opt_s() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "    0 │ 0000000000000000 - 0000000000000010 │ Section 0
    1 │ 0000000000000010 - 0000000000000025 │ Section 1
    2 │ 0000000000000025 - 000000000000006d │ Section 3
    3 │ 000000000000006d - 000000000000008f │ Section 4
    4 │ 000000000000008f - 00000000000000a6 │ Section 5
    5 │ 00000000000000a6 - 00000000000000ac │ Section 6
    6 │ 00000000000000ac - 000000000000061b │ Section 7
    7 │ 000000000000061b - 000000000000063d │ Section 4
    8 │ 000000000000063d - 0000000000000654 │ Section 5
    9 │ 0000000000000654 - 000000000000065a │ Section 6
   10 │ 000000000000065a - 0000000000000bd1 │ Section 7
   11 │ 0000000000000bd1 - 0000000000000bf3 │ Section 4
   12 │ 0000000000000bf3 - 0000000000000c0a │ Section 5
   13 │ 0000000000000c0a - 0000000000000c10 │ Section 6
   14 │ 0000000000000c10 - 000000000000118c │ Section 7
   15 │ 000000000000118c - 00000000000011ae │ Section 4
   16 │ 00000000000011ae - 00000000000011c5 │ Section 5
   17 │ 00000000000011c5 - 00000000000011cb │ Section 6
   18 │ 00000000000011cb - 000000000000173e │ Section 7
   19 │ 000000000000173e - 0000000000001760 │ Section 4
   20 │ 0000000000001760 - 0000000000001777 │ Section 5
   21 │ 0000000000001777 - 000000000000177d │ Section 6
   22 │ 000000000000177d - 0000000000001cf0 │ Section 7
   23 │ 0000000000001cf0 - 0000000000001d12 │ Section 4
   24 │ 0000000000001d12 - 0000000000001d29 │ Section 5
   25 │ 0000000000001d29 - 0000000000001d2f │ Section 6
   26 │ 0000000000001d2f - 00000000000022a4 │ Section 7
   27 │ 00000000000022a4 - 00000000000022c6 │ Section 4
   28 │ 00000000000022c6 - 00000000000022dd │ Section 5
   29 │ 00000000000022dd - 00000000000022e3 │ Section 6
   30 │ 00000000000022e3 - 000000000000284d │ Section 7
   31 │ 000000000000284d - 0000000000002851 │ Section 8
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect").arg("-s").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn inspect_with_opt_m() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "   id │    S2    S3    S4    S5    S6    S7 │ Tmpl3   Tmpl4   Tmpl5
    0 │     -     2     3     4     5     6 │ 3.0     4.0     5.200  
    1 │     -     2     7     8     9    10 │ 3.0     4.0     5.200  
    2 │     -     2    11    12    13    14 │ 3.0     4.0     5.200  
    3 │     -     2    15    16    17    18 │ 3.0     4.0     5.200  
    4 │     -     2    19    20    21    22 │ 3.0     4.0     5.200  
    5 │     -     2    23    24    25    26 │ 3.0     4.0     5.200  
    6 │     -     2    27    28    29    30 │ 3.0     4.0     5.200  
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect").arg("-m").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn inspect_with_opt_t() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let out_str = "\
3.0      - Latitude/longitude
4.0      - Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
5.200    - Run length packing with level values
";

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect").arg("-t").arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(out_str))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn inspect_with_all_opts() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect").arg(arg_path);
    let msg_no_opt = cmd.output()?.stdout;
    let msg_no_opt = format!("{}", String::from_utf8(msg_no_opt)?);

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect")
        .arg("-s")
        .arg("-m")
        .arg("-t")
        .arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::similar(msg_no_opt))
        .stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn decode_run_length_encoding() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode").arg(arg_path).arg("3");
    cmd.assert().success().stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn decode_simple_packing() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_kousa_file()?;
    let arg_path = tempfile.path();

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode").arg(arg_path).arg("3");
    cmd.assert().success().stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn decode_complex_packing() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_meps_file()?;
    let arg_path = tempfile.path();

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode").arg(arg_path).arg("2");
    cmd.assert().success().stderr(predicate::str::is_empty());

    Ok(())
}

#[test]
fn decode_run_length_packing_as_big_endian() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let dir = TempDir::new()?;
    let out_path = dir.path().join("out.bin");
    let out_path = format!("{}", out_path.display());

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg(arg_path)
        .arg("3")
        .arg("-b")
        .arg(&out_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = utils::tornado_nowcast_be_bin_bytes()?;
    let expected: Vec<_> = expected
        .chunks(4)
        .into_iter()
        .map(|b| match b {
            [0x62, 0x58, 0xd1, 0x9a] => vec![0x7f, 0xc0, 0x00, 0x00],
            b => b.to_vec(),
        })
        .flatten()
        .collect();
    let actual = utils::cat_as_bytes(&out_path)?;
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn decode_run_length_packing_as_little_endian() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let dir = TempDir::new()?;
    let out_path = dir.path().join("out.bin");
    let out_path = format!("{}", out_path.display());

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg(arg_path)
        .arg("3")
        .arg("-l")
        .arg(&out_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = utils::tornado_nowcast_le_bin_bytes()?;
    let expected: Vec<_> = expected
        .chunks(4)
        .into_iter()
        .map(|b| match b {
            [0x9a, 0xd1, 0x58, 0x62] => vec![0x00, 0x00, 0xc0, 0x7f],
            b => b.to_vec(),
        })
        .flatten()
        .collect();
    let actual = utils::cat_as_bytes(&out_path)?;
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn decode_simple_packing_as_big_endian() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_kousa_file()?;
    let arg_path = tempfile.path();

    let dir = TempDir::new()?;
    let out_path = dir.path().join("out.bin");
    let out_path = format!("{}", out_path.display());

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg(arg_path)
        .arg("3")
        .arg("-b")
        .arg(&out_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = utils::kousa_be_bin_bytes()?;
    let actual = utils::cat_as_bytes(&out_path)?;
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn decode_simple_packing_as_little_endian() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_kousa_file()?;
    let arg_path = tempfile.path();

    let dir = TempDir::new()?;
    let out_path = dir.path().join("out.bin");
    let out_path = format!("{}", out_path.display());

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg(arg_path)
        .arg("3")
        .arg("-l")
        .arg(&out_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = utils::kousa_le_bin_bytes()?;
    let actual = utils::cat_as_bytes(&out_path)?;
    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn decode_complex_packing_as_little_endian() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_meps_file()?;
    let arg_path = tempfile.path();

    let dir = TempDir::new()?;
    let out_path = dir.path().join("out.bin");
    let out_path = format!("{}", out_path.display());

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("decode")
        .arg(arg_path)
        .arg("2")
        .arg("-l")
        .arg(&out_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());

    let expected = utils::meps_le_bin_bytes()?;
    let actual = utils::cat_as_bytes(&out_path)?;
    assert_eq!(actual, expected);

    Ok(())
}

macro_rules! test_subcommands_without_args {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr(
                    predicate::str::starts_with(
                        "error: The following required arguments were not provided:",
                    )
                        .and(predicate::str::contains("USAGE:"))
                        .and(predicate::str::contains("SUBCOMMANDS:").not()),
                );

            Ok(())
        }
    )*);
}

test_subcommands_without_args! {
    (info_without_args, "info"),
    (list_without_args, "list"),
    (inspect_without_args, "inspect"),
}

macro_rules! test_subcommands_with_nonexisting_file {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let dir = TempDir::new()?;
            let file_path = dir.path().join("nosuchfile");
            let file_path = format!("{}", file_path.display());

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str).arg(file_path);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty());

            Ok(())
        }
    )*);
}

test_subcommands_with_nonexisting_file! {
    (info_with_nonexisting_file, "info"),
    (list_with_nonexisting_file, "list"),
    (inspect_with_nonexisting_file, "inspect"),
}

macro_rules! test_subcommands_with_non_grib {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let tempfile = utils::non_grib_file()?;
            let arg_path = tempfile.path();

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str).arg(arg_path);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::similar("Not GRIB data\n"));

            Ok(())
        }
    )*);
}

test_subcommands_with_non_grib! {
    (info_with_non_grib, "info"),
    (list_with_non_grib, "list"),
    (inspect_with_non_grib, "inspect"),
}

macro_rules! test_subcommands_with_too_small_file {
    ($(($name:ident, $str:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let tempfile = utils::too_small_file()?;
            let arg_path = tempfile.path();

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg($str).arg(arg_path);
            cmd.assert()
                .failure()
                .stdout(predicate::str::is_empty())
                .stderr(predicate::str::similar(
                    "Error in checking file type: failed to fill whole buffer\n",
                ));

            Ok(())
        }
    )*);
}

test_subcommands_with_too_small_file! {
    (info_with_too_small_file, "info"),
    (list_with_too_small_file, "list"),
    (inspect_with_too_small_file, "inspect"),
}
