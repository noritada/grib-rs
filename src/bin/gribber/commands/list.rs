use clap::{App, Arg, ArgMatches, SubCommand};
use std::fmt::{self, Display, Formatter};

use grib::context::SubMessageIterator;

use crate::cli;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("list")
        .about("Lists contained data")
        .arg(Arg::with_name("file").required(true))
}

pub fn exec(args: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = args.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;

    let view = ListView::new(grib.submessages());
    print!("{}", view);

    Ok(())
}

struct ListView<'i> {
    data: SubMessageIterator<'i>,
}

impl<'i> ListView<'i> {
    fn new(data: SubMessageIterator<'i>) -> Self {
        Self { data: data }
    }
}

impl<'i> Display for ListView<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, submessage) in self.data.clone().enumerate() {
            write!(f, "{}\n{}\n", i, submessage.describe())?;
        }
        Ok(())
    }
}
