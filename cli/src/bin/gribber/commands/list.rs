use clap::{arg, ArgMatches, Command};
use console::{Style, Term};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use grib::codetables::{CodeTable4_2, CodeTable4_3, Lookup};
use grib::context::SubmessageIterator;

use crate::cli;

pub fn cli() -> Command<'static> {
    Command::new("list")
        .about("List surfaces contained in the data")
        .arg(arg!(-d --dump "Show details of each data"))
        .arg(arg!(<FILE> "Target file").value_parser(clap::value_parser!(PathBuf)))
}

pub fn exec(args: &ArgMatches) -> anyhow::Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;

    let mode = if args.contains_id("dump") {
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
    data: SubmessageIterator<'i>,
    mode: ListViewMode,
}

impl<'i> ListView<'i> {
    fn new(data: SubmessageIterator<'i>, mode: ListViewMode) -> Self {
        Self { data, mode }
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
                (unit_height + 2) * len - 1
            }
        }
    }
}

impl<'i> Display for ListView<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let entries = self.data.clone();
        match self.mode {
            ListViewMode::OneLine => {
                let header = format!(
                    "{:>8} │ {:<31} {:<18} {:>14} {:>17} {:>17} | {:>21}\n",
                    "id",
                    "Parameter",
                    "Generating process",
                    "Forecast time",
                    "1st fixed surface",
                    "2nd fixed surface",
                    "#points (nan/total)"
                );
                let style = Style::new().bold();
                write!(f, "{}", style.apply_to(header))?;

                for (i, submessage) in entries {
                    let id = format!("{}.{}", i.0, i.1);
                    let prod_def = submessage.prod_def();
                    let category = prod_def
                        .parameter_category()
                        .zip(prod_def.parameter_number())
                        .map(|(c, n)| {
                            CodeTable4_2::new(submessage.indicator().discipline, c)
                                .lookup(usize::from(n))
                                .to_string()
                        })
                        .unwrap_or_default();
                    let generating_process = prod_def
                        .generating_process()
                        .map(|v| CodeTable4_3.lookup(usize::from(v)).to_string())
                        .unwrap_or_default();
                    let forecast_time = prod_def
                        .forecast_time()
                        .map(|ft| ft.to_string())
                        .unwrap_or_default();
                    let surfaces = prod_def
                        .fixed_surfaces()
                        .map(|(first, second)| {
                            (first.value().to_string(), second.value().to_string())
                        })
                        .unwrap_or((String::new(), String::new()));
                    let num_grid_points = submessage.grid_def().num_points();
                    let num_points_represented = submessage.repr_def().num_points();
                    writeln!(
                        f,
                        "{:>8} │ {:<31} {:<18} {:>14} {:>17} {:>17} | {:>10}/{:>10}",
                        id,
                        category,
                        generating_process,
                        forecast_time,
                        surfaces.0,
                        surfaces.1,
                        num_grid_points - num_points_represented,
                        num_grid_points
                    )?;
                }
            }
            ListViewMode::Dump => {
                for (i, submessage) in entries {
                    let id = format!("{}.{}", i.0, i.1);
                    write!(f, "{}\n{}\n", id, submessage.describe())?;
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
