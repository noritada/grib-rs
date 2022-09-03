use crate::{utils, CMD_NAME};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

macro_rules! test_display_with_no_options {
    ($(($name:ident, $input:expr, $args:expr, $expected_stdout:expr),)*) => ($(
        #[test]
        fn $name() -> Result<(), Box<dyn std::error::Error>> {
            let input = $input;

            let mut cmd = Command::cargo_bin(CMD_NAME)?;
            cmd.arg("list").args($args).arg(input.path());
            cmd.assert()
                .success()
                .stdout(predicate::str::diff($expected_stdout))
                .stderr(predicate::str::is_empty());

            Ok(())
        }
    )*);
}

test_display_with_no_options! {
    (
        displaying_grib2_with_multiple_submessages_without_nan_values,
        utils::testdata::grib2::jma_tornado_nowcast()?,
        Vec::<&str>::new(),
        "      id │ Parameter                       Generating process  Forecast time 1st fixed surface 2nd fixed surface |   #points (nan/total)
     0.0 │ code '0' is not implemented     Analysis                    0 [m]               NaN               NaN |          0/     86016
     0.1 │ code '0' is not implemented     Forecast                   10 [m]               NaN               NaN |          0/     86016
     0.2 │ code '0' is not implemented     Forecast                   20 [m]               NaN               NaN |          0/     86016
     0.3 │ code '0' is not implemented     Forecast                   30 [m]               NaN               NaN |          0/     86016
     0.4 │ code '0' is not implemented     Forecast                   40 [m]               NaN               NaN |          0/     86016
     0.5 │ code '0' is not implemented     Forecast                   50 [m]               NaN               NaN |          0/     86016
     0.6 │ code '0' is not implemented     Forecast                   60 [m]               NaN               NaN |          0/     86016
"
    ),
    (
        displaying_grib2_with_multiple_submessages_with_nan_values,
        utils::testdata::grib2::jma_msmguid()?,
        Vec::<&str>::new(),
        "      id │ Parameter                       Generating process  Forecast time 1st fixed surface 2nd fixed surface |   #points (nan/total)
     0.0 │ code '192' is not implemented   Forecast                    0 [h]               NaN               NaN |     106575/    268800
     0.1 │ Total precipitation rate        Forecast                    0 [h]               NaN               NaN |     106575/    268800
     0.2 │ code '192' is not implemented   Forecast                    3 [h]               NaN               NaN |     106575/    268800
     0.3 │ Total precipitation rate        Forecast                    3 [h]               NaN               NaN |     106575/    268800
     0.4 │ code '192' is not implemented   Forecast                    6 [h]               NaN               NaN |     106575/    268800
     0.5 │ Total precipitation rate        Forecast                    6 [h]               NaN               NaN |     106575/    268800
     0.6 │ Total precipitation rate        Forecast                    3 [h]               NaN               NaN |     106575/    268800
     0.7 │ code '192' is not implemented   Forecast                    9 [h]               NaN               NaN |     106575/    268800
     0.8 │ Total precipitation rate        Forecast                    9 [h]               NaN               NaN |     106575/    268800
     0.9 │ code '192' is not implemented   Forecast                   12 [h]               NaN               NaN |     106575/    268800
    0.10 │ Total precipitation rate        Forecast                   12 [h]               NaN               NaN |     106575/    268800
    0.11 │ Total precipitation rate        Forecast                    9 [h]               NaN               NaN |     106575/    268800
    0.12 │ code '192' is not implemented   Forecast                   15 [h]               NaN               NaN |     106575/    268800
    0.13 │ Total precipitation rate        Forecast                   15 [h]               NaN               NaN |     106575/    268800
    0.14 │ code '192' is not implemented   Forecast                   18 [h]               NaN               NaN |     106575/    268800
    0.15 │ Total precipitation rate        Forecast                   18 [h]               NaN               NaN |     106575/    268800
    0.16 │ Total precipitation rate        Forecast                   15 [h]               NaN               NaN |     106575/    268800
    0.17 │ code '192' is not implemented   Forecast                   21 [h]               NaN               NaN |     106575/    268800
    0.18 │ Total precipitation rate        Forecast                   21 [h]               NaN               NaN |     106575/    268800
    0.19 │ code '192' is not implemented   Forecast                   24 [h]               NaN               NaN |     106575/    268800
    0.20 │ Total precipitation rate        Forecast                   24 [h]               NaN               NaN |     106575/    268800
    0.21 │ Total precipitation rate        Forecast                   21 [h]               NaN               NaN |     106575/    268800
    0.22 │ code '192' is not implemented   Forecast                   27 [h]               NaN               NaN |     106575/    268800
    0.23 │ Total precipitation rate        Forecast                   27 [h]               NaN               NaN |     106575/    268800
    0.24 │ code '192' is not implemented   Forecast                   30 [h]               NaN               NaN |     106575/    268800
    0.25 │ Total precipitation rate        Forecast                   30 [h]               NaN               NaN |     106575/    268800
    0.26 │ Total precipitation rate        Forecast                   27 [h]               NaN               NaN |     106575/    268800
    0.27 │ code '192' is not implemented   Forecast                   33 [h]               NaN               NaN |     106575/    268800
    0.28 │ Total precipitation rate        Forecast                   33 [h]               NaN               NaN |     106575/    268800
    0.29 │ code '192' is not implemented   Forecast                   36 [h]               NaN               NaN |     106575/    268800
    0.30 │ Total precipitation rate        Forecast                   36 [h]               NaN               NaN |     106575/    268800
    0.31 │ Total precipitation rate        Forecast                   33 [h]               NaN               NaN |     106575/    268800
    0.32 │ Thunderstorm probability        Forecast                    0 [h]               NaN               NaN |      14446/     17061
    0.33 │ Thunderstorm probability        Forecast                    3 [h]               NaN               NaN |      14446/     17061
    0.34 │ Thunderstorm probability        Forecast                    6 [h]               NaN               NaN |      14446/     17061
    0.35 │ Thunderstorm probability        Forecast                    9 [h]               NaN               NaN |      14446/     17061
    0.36 │ Thunderstorm probability        Forecast                   12 [h]               NaN               NaN |      14446/     17061
    0.37 │ Thunderstorm probability        Forecast                   15 [h]               NaN               NaN |      14446/     17061
    0.38 │ Thunderstorm probability        Forecast                   18 [h]               NaN               NaN |      14446/     17061
    0.39 │ Thunderstorm probability        Forecast                   21 [h]               NaN               NaN |      14446/     17061
    0.40 │ Thunderstorm probability        Forecast                   24 [h]               NaN               NaN |      14446/     17061
    0.41 │ Thunderstorm probability        Forecast                   27 [h]               NaN               NaN |      14446/     17061
    0.42 │ Thunderstorm probability        Forecast                   30 [h]               NaN               NaN |      14446/     17061
    0.43 │ Thunderstorm probability        Forecast                   33 [h]               NaN               NaN |      14446/     17061
    0.44 │ Thunderstorm probability        Forecast                   36 [h]               NaN               NaN |      14446/     17061
"
    ),
    (
        displaying_grib2_with_multiple_messages,
        utils::testdata::grib2::multi_message_data(3)?,
        Vec::<&str>::new(),
        "      id │ Parameter                       Generating process  Forecast time 1st fixed surface 2nd fixed surface |   #points (nan/total)
     0.0 │ Total precipitation rate        Forecast                    0 [m]                 0               NaN |          0/   2949120
     1.0 │ Total precipitation rate        Forecast                    0 [m]                 0               NaN |          0/   2949120
     2.0 │ Total precipitation rate        Forecast                    0 [m]                 0               NaN |          0/   2949120
"
    ),
    (
        displaying_grib2_with_multiple_submessages_with_opt_d,
        utils::testdata::grib2::jma_tornado_nowcast()?,
        vec!["-d"],
        "\
0.0
Grid:                                   Latitude/longitude
  Number of points:                     86016
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Analysis
  Forecast Time:                        0
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     Missing
  1st Scaled Value:                     Missing
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Run length packing with level values
  Number of represented values:         86016

0.1
Grid:                                   Latitude/longitude
  Number of points:                     86016
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        10
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     Missing
  1st Scaled Value:                     Missing
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Run length packing with level values
  Number of represented values:         86016

0.2
Grid:                                   Latitude/longitude
  Number of points:                     86016
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        20
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     Missing
  1st Scaled Value:                     Missing
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Run length packing with level values
  Number of represented values:         86016

0.3
Grid:                                   Latitude/longitude
  Number of points:                     86016
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        30
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     Missing
  1st Scaled Value:                     Missing
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Run length packing with level values
  Number of represented values:         86016

0.4
Grid:                                   Latitude/longitude
  Number of points:                     86016
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        40
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     Missing
  1st Scaled Value:                     Missing
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Run length packing with level values
  Number of represented values:         86016

0.5
Grid:                                   Latitude/longitude
  Number of points:                     86016
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        50
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     Missing
  1st Scaled Value:                     Missing
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Run length packing with level values
  Number of represented values:         86016

0.6
Grid:                                   Latitude/longitude
  Number of points:                     86016
Product:                                Analysis or forecast at a horizontal level or in a horizontal layer at a point in time
  Parameter Category:                   code '193' is not implemented
  Parameter:                            code '0' is not implemented
  Generating Proceess:                  Forecast
  Forecast Time:                        60
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     Missing
  1st Scaled Value:                     Missing
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Run length packing with level values
  Number of represented values:         86016

"
    ),
    (
        displaying_grib2_with_multiple_messages_with_opt_d,
        utils::testdata::grib2::multi_message_data(3)?,
        vec!["-d"],
        "\
0.0
Grid:                                   General unstructured grid
  Number of points:                     2949120
Product:                                Average, accumulation, extreme values or other statistically processed values at a horizontal level or in a horizontal layer in a continuous or non-continuous time interval
  Parameter Category:                   Moisture
  Parameter:                            Total precipitation rate
  Generating Proceess:                  Forecast
  Forecast Time:                        0
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     0
  1st Scaled Value:                     0
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Grid point data - simple packing
  Number of represented values:         2949120

1.0
Grid:                                   General unstructured grid
  Number of points:                     2949120
Product:                                Average, accumulation, extreme values or other statistically processed values at a horizontal level or in a horizontal layer in a continuous or non-continuous time interval
  Parameter Category:                   Moisture
  Parameter:                            Total precipitation rate
  Generating Proceess:                  Forecast
  Forecast Time:                        0
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     0
  1st Scaled Value:                     0
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Grid point data - simple packing
  Number of represented values:         2949120

2.0
Grid:                                   General unstructured grid
  Number of points:                     2949120
Product:                                Average, accumulation, extreme values or other statistically processed values at a horizontal level or in a horizontal layer in a continuous or non-continuous time interval
  Parameter Category:                   Moisture
  Parameter:                            Total precipitation rate
  Generating Proceess:                  Forecast
  Forecast Time:                        0
  Forecast Time Unit:                   Minute
  1st Fixed Surface Type:               Ground or water surface
  1st Scale Factor:                     0
  1st Scaled Value:                     0
  2nd Fixed Surface Type:               code '255' is not implemented
  2nd Scale Factor:                     Missing
  2nd Scaled Value:                     Missing
Data Representation:                    Grid point data - simple packing
  Number of represented values:         2949120

"
    ),
}
