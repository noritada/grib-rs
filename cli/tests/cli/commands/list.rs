use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use crate::{CMD_NAME, utils};

crate::commands::test_simple_display! {
    (
        displaying_grib2_with_multiple_submessages_without_nan_values,
        "list",
        utils::testdata::grib2::jma_tornado_nowcast()?,
        Vec::<&str>::new(),
        "      id │ Parameter                       Generating process  Forecast time                 1st fixed surface                 2nd fixed surface │   #points (nan/total) grid type
     0.0 │ code '0' is not implemented     Analysis                    0 [m]                               NaN                               NaN │          0/     86016 regular_ll           
     0.1 │ code '0' is not implemented     Forecast                   10 [m]                               NaN                               NaN │          0/     86016 regular_ll           
     0.2 │ code '0' is not implemented     Forecast                   20 [m]                               NaN                               NaN │          0/     86016 regular_ll           
     0.3 │ code '0' is not implemented     Forecast                   30 [m]                               NaN                               NaN │          0/     86016 regular_ll           
     0.4 │ code '0' is not implemented     Forecast                   40 [m]                               NaN                               NaN │          0/     86016 regular_ll           
     0.5 │ code '0' is not implemented     Forecast                   50 [m]                               NaN                               NaN │          0/     86016 regular_ll           
     0.6 │ code '0' is not implemented     Forecast                   60 [m]                               NaN                               NaN │          0/     86016 regular_ll           
",
    ),
    (
        displaying_grib2_with_multiple_submessages_with_nan_values,
        "list",
        utils::testdata::grib2::jma_msmguid()?,
        Vec::<&str>::new(),
        "      id │ Parameter                       Generating process  Forecast time                 1st fixed surface                 2nd fixed surface │   #points (nan/total) grid type
     0.0 │ code '192' is not implemented   Forecast                    0 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.1 │ Total precipitation rate        Forecast                    0 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.2 │ code '192' is not implemented   Forecast                    3 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.3 │ Total precipitation rate        Forecast                    3 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.4 │ code '192' is not implemented   Forecast                    6 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.5 │ Total precipitation rate        Forecast                    6 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.6 │ Total precipitation rate        Forecast                    3 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.7 │ code '192' is not implemented   Forecast                    9 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.8 │ Total precipitation rate        Forecast                    9 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
     0.9 │ code '192' is not implemented   Forecast                   12 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.10 │ Total precipitation rate        Forecast                   12 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.11 │ Total precipitation rate        Forecast                    9 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.12 │ code '192' is not implemented   Forecast                   15 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.13 │ Total precipitation rate        Forecast                   15 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.14 │ code '192' is not implemented   Forecast                   18 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.15 │ Total precipitation rate        Forecast                   18 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.16 │ Total precipitation rate        Forecast                   15 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.17 │ code '192' is not implemented   Forecast                   21 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.18 │ Total precipitation rate        Forecast                   21 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.19 │ code '192' is not implemented   Forecast                   24 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.20 │ Total precipitation rate        Forecast                   24 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.21 │ Total precipitation rate        Forecast                   21 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.22 │ code '192' is not implemented   Forecast                   27 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.23 │ Total precipitation rate        Forecast                   27 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.24 │ code '192' is not implemented   Forecast                   30 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.25 │ Total precipitation rate        Forecast                   30 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.26 │ Total precipitation rate        Forecast                   27 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.27 │ code '192' is not implemented   Forecast                   33 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.28 │ Total precipitation rate        Forecast                   33 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.29 │ code '192' is not implemented   Forecast                   36 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.30 │ Total precipitation rate        Forecast                   36 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.31 │ Total precipitation rate        Forecast                   33 [h]                               NaN                               NaN │     106575/    268800 regular_ll           
    0.32 │ Thunderstorm probability        Forecast                    0 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.33 │ Thunderstorm probability        Forecast                    3 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.34 │ Thunderstorm probability        Forecast                    6 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.35 │ Thunderstorm probability        Forecast                    9 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.36 │ Thunderstorm probability        Forecast                   12 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.37 │ Thunderstorm probability        Forecast                   15 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.38 │ Thunderstorm probability        Forecast                   18 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.39 │ Thunderstorm probability        Forecast                   21 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.40 │ Thunderstorm probability        Forecast                   24 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.41 │ Thunderstorm probability        Forecast                   27 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.42 │ Thunderstorm probability        Forecast                   30 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.43 │ Thunderstorm probability        Forecast                   33 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
    0.44 │ Thunderstorm probability        Forecast                   36 [h]                               NaN                               NaN │      14446/     17061 regular_ll           
",
    ),
    (
        displaying_grib2_with_multiple_messages,
        "list",
        utils::testdata::grib2::multi_message_data(3)?,
        Vec::<&str>::new(),
        r#"      id │ Parameter                       Generating process  Forecast time                 1st fixed surface                 2nd fixed surface │   #points (nan/total) grid type
     0.0 │ Total precipitation rate        Forecast                    0 [m]                                 0                               NaN │          0/   2949120 unknown (template 101)
     1.0 │ Total precipitation rate        Forecast                    0 [m]                                 0                               NaN │          0/   2949120 unknown (template 101)
     2.0 │ Total precipitation rate        Forecast                    0 [m]                                 0                               NaN │          0/   2949120 unknown (template 101)
"#,
    ),
}
