use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use clap::{arg, ArgAction, ArgMatches, Command};
use console::Style;
use grib::{
    codetables::{CodeTable4_2, CodeTable4_3, Lookup},
    SubmessageIterator,
};

use crate::cli;

pub fn cli() -> Command {
    Command::new(crate::cli::module_component!())
        .about("List layers contained in the data")
        .arg(arg!(-d --dump "Show details of each data").action(ArgAction::SetTrue))
        .arg(
            arg!(<FILE> "Target file name (or a single dash (`-`) for standard input)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
}

pub fn exec(args: &ArgMatches) -> anyhow::Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;

    let mode = if args.get_flag("dump") {
        ListViewMode::Dump
    } else {
        ListViewMode::OneLine
    };
    let view = ListView::new(grib.submessages(), mode);
    cli::display_in_pager(view);

    Ok(())
}

struct ListView<'i, R> {
    data: SubmessageIterator<'i, R>,
    mode: ListViewMode,
}

impl<'i, R> ListView<'i, R> {
    fn new(data: SubmessageIterator<'i, R>, mode: ListViewMode) -> Self {
        Self { data, mode }
    }
}

impl<R> cli::PredictableNumLines for ListView<'_, R> {
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

impl<R> Display for ListView<'_, R> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let entries = &self.data;
        match self.mode {
            ListViewMode::OneLine => {
                let header = format!(
                    "{:>8} │ {:<31} {:<18} {:>14} {:>33} {:>33} │ {:>21} {:<21}",
                    "id",
                    "Parameter",
                    "Generating process",
                    "Forecast time",
                    "1st fixed surface",
                    "2nd fixed surface",
                    "#points (nan/total)",
                    "grid type",
                );
                let style = Style::new().bold();
                writeln!(f, "{}", style.apply_to(header.trim_end()))?;

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
                        .map(|(first, second)| (format_surface(&first), format_surface(&second)))
                        .unwrap_or((String::new(), String::new()));
                    let grid_def = submessage.grid_def();
                    let num_grid_points = grid_def.num_points();
                    let num_points_represented = submessage.repr_def().num_points();
                    let grid_type = grib::GridDefinitionTemplateValues::try_from(grid_def)
                        .map(|def| Cow::from(def.short_name()))
                        .unwrap_or_else(|_| {
                            Cow::from(format!("unknown (template {})", grid_def.grid_tmpl_num()))
                        });
                    writeln!(
                        f,
                        "{:>8} │ {:<31} {:<18} {:>14} {:>33} {:>33} │ {:>10}/{:>10} {:<21}",
                        id,
                        category,
                        generating_process,
                        forecast_time,
                        surfaces.0,
                        surfaces.1,
                        num_grid_points - num_points_represented,
                        num_grid_points,
                        grid_type,
                    )?;
                }
            }
            ListViewMode::Dump => {
                for (i, submessage) in entries {
                    let id = format!("{}.{}", i.0, i.1);
                    write!(f, "{id}\n{}\n", submessage.describe())?;
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

fn format_surface(surface: &grib::FixedSurface) -> String {
    let value = surface.value();
    let unit = surface
        .unit()
        .map(|s| format!(" [{s}]"))
        .unwrap_or_default();
    format!("{value}{unit}")
}
