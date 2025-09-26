use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use crate::{CMD_NAME, utils};

crate::commands::test_simple_display! {
    (
        display_of_single_message_data,
        "info",
        utils::testdata::grib2::jma_tornado_nowcast()?,
        Vec::<&str>::new(),
        "\
Message 0

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

"
    ),
    (
        display_of_multi_message_data,
        "info",
        utils::testdata::grib2::multi_message_data(3)?,
        Vec::<&str>::new(),
        "\
Message 0

    Discipline:                             Meteorological products
    Total Length:                           193
    Originating/generating centre:          Offenbach (RSMC)
    Originating/generating sub-centre:      255
    GRIB Master Tables Version Number:      19 (3 May 2017)
    GRIB Local Tables Version Number:       1 (Number of local tables version used)
    Significance of Reference Time:         Start of forecast
    Reference time of data:                 2021-11-20 18:00:00 UTC
    Production status of processed data:    Operational products
    Type of processed data:                 Forecast products

Message 1

    Discipline:                             Meteorological products
    Total Length:                           193
    Originating/generating centre:          Offenbach (RSMC)
    Originating/generating sub-centre:      255
    GRIB Master Tables Version Number:      19 (3 May 2017)
    GRIB Local Tables Version Number:       1 (Number of local tables version used)
    Significance of Reference Time:         Start of forecast
    Reference time of data:                 2021-11-20 18:00:00 UTC
    Production status of processed data:    Operational products
    Type of processed data:                 Forecast products

Message 2

    Discipline:                             Meteorological products
    Total Length:                           193
    Originating/generating centre:          Offenbach (RSMC)
    Originating/generating sub-centre:      255
    GRIB Master Tables Version Number:      19 (3 May 2017)
    GRIB Local Tables Version Number:       1 (Number of local tables version used)
    Significance of Reference Time:         Start of forecast
    Reference time of data:                 2021-11-20 18:00:00 UTC
    Production status of processed data:    Operational products
    Type of processed data:                 Forecast products

"
    ),
}
