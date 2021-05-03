use clap::{App, Arg, ArgMatches, SubCommand};
use console::Term;
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

    let view = ListView::new(grib.submessages(), ListViewMode::Dump);

    let term = Term::stdout();
    let (height, _width) = term.size();
    if view.num_lines() > height.into() {
        cli::start_pager();
    }

    print!("{}", view);

    Ok(())
}

struct ListView<'i> {
    data: SubMessageIterator<'i>,
    mode: ListViewMode,
}

impl<'i> ListView<'i> {
    fn new(data: SubMessageIterator<'i>, mode: ListViewMode) -> Self {
        Self {
            data: data,
            mode: mode,
        }
    }

    fn num_lines(&self) -> usize {
        match self.mode {
            ListViewMode::Dump => {
                let unit_height = 8; // lines of output from SubMessage.describe(), hard-coded as of now
                let (len, _) = self.data.size_hint();
                let total_height = (unit_height + 2) * len - 1;
                total_height
            }
        }
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

enum ListViewMode {
    Dump,
}
