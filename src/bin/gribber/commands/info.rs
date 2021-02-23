use clap::{App, Arg, ArgMatches, SubCommand};
use std::fmt::{self, Display, Formatter};

use grib::codetables::{
    lookup_table, CODE_TABLE_0_0, CODE_TABLE_1_1, CODE_TABLE_1_2, CODE_TABLE_1_3, CODE_TABLE_1_4,
    COMMON_CODE_TABLE_00, COMMON_CODE_TABLE_11,
};
use grib::context::{Identification, Indicator};

use crate::cli;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("info")
        .about("Shows identification information")
        .arg(Arg::with_name("file").required(true))
}

pub fn exec(args: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = args.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;
    let info = InfoItem::new(grib.info()?);
    print!("{}", info);
    Ok(())
}

struct InfoItem<'i> {
    indicator: &'i Indicator,
    identification: &'i Identification,
}

impl<'i> InfoItem<'i> {
    fn new(info: (&'i Indicator, &'i Identification)) -> Self {
        Self {
            indicator: info.0,
            identification: info.1,
        }
    }
}

impl<'i> Display for InfoItem<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "\
Discipline:                             {}
Total Length:                           {}
Originating/generating centre:          {}
Originating/generating sub-centre:      {}
GRIB Master Tables Version Number:      {} ({})
GRIB Local Tables Version Number:       {}
Significance of Reference Time:         {}
Reference time of data:                 {}
Production status of processed data:    {}
Type of processed data:                 {}
",
            lookup_table(CODE_TABLE_0_0, self.indicator.discipline as usize),
            self.indicator.total_length,
            lookup_table(COMMON_CODE_TABLE_11, self.identification.centre_id as usize),
            self.identification.subcentre_id,
            self.identification.master_table_version,
            lookup_table(
                COMMON_CODE_TABLE_00,
                self.identification.master_table_version as usize
            ),
            lookup_table(
                CODE_TABLE_1_1,
                self.identification.local_table_version as usize
            ),
            lookup_table(
                CODE_TABLE_1_2,
                self.identification.ref_time_significance as usize
            ),
            self.identification.ref_time,
            lookup_table(CODE_TABLE_1_3, self.identification.prod_status as usize),
            lookup_table(CODE_TABLE_1_4, self.identification.data_type as usize)
        )
    }
}
