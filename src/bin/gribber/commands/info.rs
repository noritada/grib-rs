use clap::{arg, ArgMatches, Command};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use grib::codetables::{
    CodeTable0_0, CodeTable1_1, CodeTable1_2, CodeTable1_3, CodeTable1_4, CommonCodeTable00,
    CommonCodeTable11, Lookup,
};
use grib::datatypes::{Identification, Indicator};

use crate::cli;

pub fn cli() -> Command<'static> {
    Command::new("info")
        .about("Show identification information")
        .arg(arg!(<FILE> "Target file").value_parser(clap::value_parser!(PathBuf)))
}

pub fn exec(args: &ArgMatches) -> anyhow::Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;
    let (indicator, identification) = grib.info()?;
    let info = InfoView(indicator, identification);
    print!("{}", info);
    Ok(())
}

struct InfoView<'i>(&'i Indicator, &'i Identification);

impl<'i> Display for InfoView<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self(indicator, identification) = self;
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
            CodeTable0_0.lookup(indicator.discipline as usize),
            indicator.total_length,
            CommonCodeTable11.lookup(identification.centre_id() as usize),
            identification.subcentre_id(),
            identification.master_table_version(),
            CommonCodeTable00.lookup(identification.master_table_version() as usize),
            identification.local_table_version(),
            CodeTable1_1.lookup(identification.local_table_version() as usize),
            CodeTable1_2.lookup(identification.ref_time_significance() as usize),
            identification.ref_time(),
            CodeTable1_3.lookup(identification.prod_status() as usize),
            CodeTable1_4.lookup(identification.data_type() as usize)
        )
    }
}
