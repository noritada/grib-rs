use clap::{App, Arg, ArgMatches, SubCommand};
use console::{Style, Term};
use std::fmt::{self, Display, Formatter};

use grib::codetables::{CodeTable4_2, CodeTable4_3, CodeTable4_4, Lookup};
use grib::context::SubMessageIterator;

use crate::cli;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("list")
        .about("Lists surfaces contained in the data")
        .arg(
            Arg::with_name("dump")
                .help("Shows details of each data")
                .short("d")
                .long("dump"),
        )
        .arg(Arg::with_name("file").required(true))
}

pub fn exec(args: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = args.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;

    let mode = if args.is_present("dump") {
        ListViewMode::Dump
    } else {
        ListViewMode::OneLine
    };
    let view = ListView::new(grib.submessages(), mode);

    let user_attended = console::user_attended();

    let term = Term::stdout();
    let (height, _width) = term.size();
    if view.num_lines() > height.into() {
        cli::start_pager();
    }

    if user_attended {
        console::set_colors_enabled(true);
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
            ListViewMode::OneLine => {
                let header_height = 1;
                let (len, _) = self.data.size_hint();
                header_height + len
            }
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
        match self.mode {
            ListViewMode::OneLine => {
                let header = format!(
                    "{:>5} │ {:<31} {:<18} {:>14} {:>17} {:>17}\n",
                    "id",
                    "Parameter",
                    "Generating process",
                    "Forecast time",
                    "1st fixed surface",
                    "2nd fixed surface"
                );
                let style = Style::new().bold();
                write!(f, "{}", style.apply_to(header))?;

                for (i, submessage) in self.data.clone().enumerate() {
                    let prod_def = submessage.prod_def();
                    let category = prod_def
                        .parameter_category()
                        .zip(prod_def.parameter_number())
                        .map(|(c, n)| {
                            CodeTable4_2::new(submessage.indicator().discipline, c)
                                .lookup(usize::from(n))
                                .to_string()
                        })
                        .unwrap_or(String::new());
                    let generating_process = prod_def
                        .generating_process()
                        .map(|v| CodeTable4_3.lookup(usize::from(v)).to_string())
                        .unwrap_or(String::new());
                    let forecast_time = prod_def
                        .forecast_time()
                        .map(|(unit, v)| {
                            let unit = CodeTable4_4.lookup(usize::from(unit));
                            let value = v;
                            format!("{} {}", value, unit)
                        })
                        .unwrap_or(String::new());
                    let surfaces = prod_def
                        .fixed_surfaces()
                        .map(|(first, second)| {
                            (first.value().to_string(), second.value().to_string())
                        })
                        .unwrap_or((String::new(), String::new()));
                    write!(
                        f,
                        "{:>5} │ {:<31} {:<18} {:>14} {:>17} {:>17}\n",
                        i, category, generating_process, forecast_time, surfaces.0, surfaces.1,
                    )?;
                }
            }
            ListViewMode::Dump => {
                for (i, submessage) in self.data.clone().enumerate() {
                    write!(f, "{}\n{}\n", i, submessage.describe())?;
                }
            }
        }

        Ok(())
    }
}

enum ListViewMode {
    OneLine,
    Dump,
}
