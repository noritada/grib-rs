use clap::{arg, ArgMatches, Command};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use grib::codetables::{
    CodeTable0_0, CodeTable1_1, CodeTable1_2, CodeTable1_3, CodeTable1_4, CommonCodeTable00,
    CommonCodeTable11, Lookup,
};
use grib::context::SectionBody;
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
    let iter = grib
        .iter()
        .filter(|((_, submessage_part), _)| *submessage_part == 0);
    for (message_index, submessage) in iter {
        if let (Some(SectionBody::Section0(sect0_body)), Some(SectionBody::Section1(sect1_body))) =
            (&submessage.0.body.body, &submessage.1.body.body)
        {
            print!("{}", InfoView(message_index.0, sect0_body, sect1_body));
        }
    }
    Ok(())
}

struct InfoView<'i>(usize, &'i Indicator, &'i Identification);

impl<'i> Display for InfoView<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self(index, indicator, identification) = self;
        write!(
            f,
            "\
Message {}

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
            index,
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
