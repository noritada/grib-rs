use crate::{utils, CMD_NAME};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

macro_rules! test_display_with_arguments {
    ($(($name:ident, $input:expr, $args:expr, $expected:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;
            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("inspect").args($args).arg(input.path());
            cmd.assert()
                .success()
                .stdout(predicate::str::diff($expected))
                .stderr(predicate::str::is_empty());

            Ok(())
        }
    )*);
}

test_display_with_arguments! {
    (
        display_without_options,
        utils::jma_tornado_nowcast_file()?,
        Vec::<&str>::new(),
        "\
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
     0.0 │     -     2     3     4     5     6 │ 3.0     4.0     5.200  
     0.1 │     -     2     7     8     9    10 │ 3.0     4.0     5.200  
     0.2 │     -     2    11    12    13    14 │ 3.0     4.0     5.200  
     0.3 │     -     2    15    16    17    18 │ 3.0     4.0     5.200  
     0.4 │     -     2    19    20    21    22 │ 3.0     4.0     5.200  
     0.5 │     -     2    23    24    25    26 │ 3.0     4.0     5.200  
     0.6 │     -     2    27    28    29    30 │ 3.0     4.0     5.200  

Templates:
3.0      - Latitude/longitude
4.0      - Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
5.200    - Run length packing with level values
"
    ),
    (
        display_with_opt_s,
        utils::jma_tornado_nowcast_file()?,
        vec!["-s"],
        "    0 │ 0000000000000000 - 0000000000000010 │ Section 0
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
"
    ),
    (
        display_with_opt_m,
        utils::jma_tornado_nowcast_file()?,
        vec!["-m"],
        "      id │    S2    S3    S4    S5    S6    S7 │ Tmpl3   Tmpl4   Tmpl5  
     0.0 │     -     2     3     4     5     6 │ 3.0     4.0     5.200  
     0.1 │     -     2     7     8     9    10 │ 3.0     4.0     5.200  
     0.2 │     -     2    11    12    13    14 │ 3.0     4.0     5.200  
     0.3 │     -     2    15    16    17    18 │ 3.0     4.0     5.200  
     0.4 │     -     2    19    20    21    22 │ 3.0     4.0     5.200  
     0.5 │     -     2    23    24    25    26 │ 3.0     4.0     5.200  
     0.6 │     -     2    27    28    29    30 │ 3.0     4.0     5.200  
"
    ),
    (
        display_with_opt_t,
        utils::jma_tornado_nowcast_file()?,
        vec!["-t"],
        "\
3.0      - Latitude/longitude
4.0      - Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
5.200    - Run length packing with level values
"
    ),
}

#[test]
fn display_with_all_options() -> Result<(), Box<dyn std::error::Error>> {
    let tempfile = utils::jma_tornado_nowcast_file()?;
    let arg_path = tempfile.path();

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect").arg(arg_path);
    let msg_no_opt = cmd.output()?.stdout;
    let msg_no_opt = String::from_utf8(msg_no_opt)?;

    let mut cmd = Command::cargo_bin(CMD_NAME)?;
    cmd.arg("inspect")
        .arg("-s")
        .arg("-m")
        .arg("-t")
        .arg(arg_path);
    cmd.assert()
        .success()
        .stdout(predicate::str::diff(msg_no_opt))
        .stderr(predicate::str::is_empty());

    Ok(())
}
