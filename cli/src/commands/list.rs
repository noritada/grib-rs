use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use clap::{ArgMatches, Command, arg};
use console::Style;
use grib::{
    GridShortName, SubmessageIterator,
    codetables::{CodeTable4_2, CodeTable4_3, Lookup},
};

use crate::cli;

pub fn cli() -> Command {
    Command::new(crate::cli::module_component!())
        .about("List layers contained in the data")
        .arg(
            arg!(<FILE> "Target file name (or a single dash (`-`) for standard input)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
}

pub fn exec(args: &ArgMatches) -> anyhow::Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;

    let view = ListView::new(grib.submessages());
    cli::display_in_pager(view);

    Ok(())
}

struct ListView<'i, R> {
    data: SubmessageIterator<'i, R>,
}

impl<'i, R> ListView<'i, R> {
    fn new(data: SubmessageIterator<'i, R>) -> Self {
        Self { data }
    }
}

impl<R> cli::PredictableNumLines for ListView<'_, R> {
    fn num_lines(&self) -> usize {
        let header_height = 1;
        let (len, _) = self.data.size_hint();
        header_height + len
    }
}

impl<R> Display for ListView<'_, R> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let entries = &self.data;
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

        Ok(())
    }
}

fn format_surface(surface: &grib::FixedSurface) -> String {
    let value = surface.value();
    let unit = surface
        .unit()
        .map(|s| format!(" [{s}]"))
        .unwrap_or_default();
    format!("{value}{unit}")
}
