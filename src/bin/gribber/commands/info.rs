use clap::{App, Arg, ArgMatches, SubCommand};
use std::fmt::{self, Display, Formatter};

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
        write!(f, "{}\n{}\n", self.indicator, self.identification,)
    }
}
