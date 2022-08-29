use clap::{arg, ArgMatches, Command};
use console::{Style, Term};
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

use grib::context::{SectionInfo, SubMessageSection, SubmessageIterator, TemplateInfo};

use crate::cli;

pub fn cli() -> Command<'static> {
    Command::new("inspect")
        .about("Inspect and describes the data structure")
        .arg(arg!(-s --sections "Print sections constructing the GRIB message"))
        .arg(arg!(-m --submessages "Print submessages in the GRIB message"))
        .arg(arg!(-t --templates "Print templates used in the GRIB message"))
        .arg(arg!(<FILE> "Target file").value_parser(clap::value_parser!(PathBuf)))
        .after_help(
            "\
This subcommand is mainly targeted at (possible) developers and
engineers, who wants to understand the data structure for the purpose
of debugging, enhancement, and education.\
",
        )
}

pub fn exec(args: &ArgMatches) -> anyhow::Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;

    let mut view = InspectView::new();
    if args.contains_id("sections") {
        view.add(InspectItem::Sections(InspectSectionsItem::new(
            grib.sections(),
        )));
    }
    if args.contains_id("submessages") {
        view.add(InspectItem::SubMessages(InspectSubMessagesItem::new(
            grib.submessages(),
        )));
    }
    if args.contains_id("templates") {
        let tmpls = grib.list_templates();
        view.add(InspectItem::Templates(InspectTemplatesItem::new(tmpls)));
    }
    if view.items.is_empty() {
        view.add(InspectItem::Sections(InspectSectionsItem::new(
            grib.sections(),
        )));
        view.add(InspectItem::SubMessages(InspectSubMessagesItem::new(
            grib.submessages(),
        )));
        let tmpls = grib.list_templates();
        view.add(InspectItem::Templates(InspectTemplatesItem::new(tmpls)));
    }

    let user_attended = console::user_attended();

    let term = Term::stdout();
    let (height, _width) = term.size();
    if view.num_lines() > height.into() {
        cli::start_pager();
    }

    if user_attended {
        console::set_colors_enabled(true);
    }

    let with_header = view.with_headers();
    let mut items = view.items.into_iter().peekable();
    loop {
        let item = match items.next() {
            None => break,
            Some(i) => i,
        };

        if with_header {
            let yellow = Style::new().yellow().bold();
            let s = format!("{}:", item.title());
            println!("{}", yellow.apply_to(s));
        }

        match item {
            InspectItem::Sections(item) => print!("{}", item),
            InspectItem::SubMessages(item) => print!("{}", item),
            InspectItem::Templates(item) => print!("{}", item),
        }

        if items.peek().is_some() {
            println!();
        }
    }

    Ok(())
}

struct InspectView<'i> {
    items: Vec<InspectItem<'i>>,
}

impl<'i> InspectView<'i> {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn add(&mut self, item: InspectItem<'i>) {
        self.items.push(item);
    }

    fn with_headers(&self) -> bool {
        self.items.len() >= 2
    }

    fn num_lines(&self) -> usize {
        let mut count = 0;
        for item in self.items.iter() {
            if self.with_headers() {
                count += 1;
            }
            count += item.len();
        }
        count += self.items.len() - 1; // empty lines
        count
    }
}

enum InspectItem<'i> {
    Sections(InspectSectionsItem<'i>),
    SubMessages(InspectSubMessagesItem<'i>),
    Templates(InspectTemplatesItem),
}

impl<'i> InspectItem<'i> {
    fn title(&self) -> &'static str {
        match self {
            InspectItem::Sections(_) => "Sections",
            InspectItem::SubMessages(_) => "SubMessages",
            InspectItem::Templates(_) => "Templates",
        }
    }

    fn len(&self) -> usize {
        match self {
            InspectItem::Sections(item) => item.len(),
            InspectItem::SubMessages(item) => item.len(),
            InspectItem::Templates(item) => item.len(),
        }
    }
}

struct InspectSectionsItem<'i> {
    data: &'i [SectionInfo],
}

impl<'i> InspectSectionsItem<'i> {
    fn new(data: &'i [SectionInfo]) -> Self {
        Self { data }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'i> Display for InspectSectionsItem<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, sect) in self.data.iter().enumerate() {
            writeln!(
                f,
                "{:>5} │ {:016x} - {:016x} │ Section {}",
                i,
                sect.offset,
                sect.offset + sect.size,
                sect.num
            )?
        }
        Ok(())
    }
}

struct InspectSubMessagesItem<'i> {
    data: SubmessageIterator<'i>,
}

impl<'i> InspectSubMessagesItem<'i> {
    fn new(data: SubmessageIterator<'i>) -> Self {
        Self { data }
    }

    fn len(&self) -> usize {
        let (size, _) = self.data.size_hint();
        size
    }
}

impl<'i> Display for InspectSubMessagesItem<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fn format_section_index(s: &SubMessageSection) -> String {
            format!("{:>5}", s.index.to_string())
        }

        fn format_section_index_optional(section: &Option<SubMessageSection>) -> String {
            let s = match section {
                None => "-".to_string(),
                Some(id) => id.index.to_string(),
            };
            format!("{:>5}", s)
        }

        fn format_template(template: Option<TemplateInfo>) -> String {
            let s = match template {
                None => "-".to_string(),
                Some(info) => format!("{}", info),
            };
            format!("{:<7}", s)
        }

        let header = format!(
            "{:>8} │ {:>5} {:>5} {:>5} {:>5} {:>5} {:>5} │ {:<7} {:<7} {:<7}\n",
            "id", "S2", "S3", "S4", "S5", "S6", "S7", "Tmpl3", "Tmpl4", "Tmpl5",
        );
        let style = Style::new().bold();
        write!(f, "{}", style.apply_to(header))?;

        for (i, submessage) in self.data.clone() {
            let id = format!("{}.{}", i.0, i.1);
            writeln!(
                f,
                "{:>8} │ {} {} {} {} {} {} │ {} {} {}",
                id,
                format_section_index_optional(&submessage.2),
                format_section_index(&submessage.3),
                format_section_index(&submessage.4),
                format_section_index(&submessage.5),
                format_section_index(&submessage.6),
                format_section_index(&submessage.7),
                format_template(submessage.3.template_code()),
                format_template(submessage.4.template_code()),
                format_template(submessage.5.template_code()),
            )?;
        }
        Ok(())
    }
}

struct InspectTemplatesItem {
    data: Vec<TemplateInfo>,
}

impl InspectTemplatesItem {
    fn new(data: Vec<TemplateInfo>) -> Self {
        Self { data }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl Display for InspectTemplatesItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for tmpl in self.data.iter() {
            match tmpl.describe() {
                Some(s) => {
                    writeln!(f, "{:<8} - {}", tmpl.to_string(), s)?;
                }
                None => {
                    writeln!(f, "{}", tmpl)?;
                }
            }
        }
        Ok(())
    }
}
