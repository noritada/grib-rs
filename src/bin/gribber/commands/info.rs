use clap::{App, Arg, ArgMatches};
use std::fmt::{self, Display, Formatter};

use grib::codetables::{
    CodeTable0_0, CodeTable1_1, CodeTable1_2, CodeTable1_3, CodeTable1_4, CommonCodeTable00,
    CommonCodeTable11, Lookup,
};
use grib::datatypes::{Identification, Indicator};

use crate::cli;

pub fn cli() -> App<'static> {
    App::new("info")
        .about("Show identification information")
        .arg(Arg::new("file").required(true))
}

pub fn exec(args: &ArgMatches) -> Result<(), cli::CliError> {
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
GRIB Local Tables Version Number:       {} ({})
Significance of Reference Time:         {}
Reference time of data:                 {}
Production status of processed data:    {}
Type of processed data:                 {}
",
            CodeTable0_0.lookup(self.indicator.discipline as usize),
            self.indicator.total_length,
            CommonCodeTable11.lookup(self.identification.centre_id as usize),
            self.identification.subcentre_id,
            self.identification.master_table_version,
            CommonCodeTable00.lookup(self.identification.master_table_version as usize),
            self.identification.local_table_version,
            CodeTable1_1.lookup(self.identification.local_table_version as usize),
            CodeTable1_2.lookup(self.identification.ref_time_significance as usize),
            self.identification.ref_time,
            CodeTable1_3.lookup(self.identification.prod_status as usize),
            CodeTable1_4.lookup(self.identification.data_type as usize)
        )
    }
}
