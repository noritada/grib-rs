use clap::{App, Arg, ArgMatches, SubCommand};
use console::{Style, Term};
use std::fmt::{self, Display, Formatter};

use grib::context::{SectionInfo, SubMessageIndex, TemplateInfo};

use crate::cli;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("inspect")
        .about("Inspects and describes the data structure")
        .arg(
            Arg::with_name("sections")
                .help("Prints sections constructing the GRIB message")
                .short("s")
                .long("sections"),
        )
        .arg(
            Arg::with_name("submessages")
                .help("Prints submessages in the GRIB message")
                .short("m")
                .long("submessages"),
        )
        .arg(
            Arg::with_name("templates")
                .help("Prints templates used in the GRIB message")
                .short("t")
                .long("templates"),
        )
        .arg(Arg::with_name("file").required(true))
        .after_help(
            "\
This subcommand is mainly targeted at (possible) developers and
engineers, who wants to understand the data structure for the purpose
of debugging, enhancement, and education.\
",
        )
}

pub fn exec(args: &ArgMatches<'static>) -> Result<(), cli::CliError> {
    let file_name = args.value_of("file").unwrap();
    let grib = cli::grib(file_name)?;

    let mut view = InspectView::new();
    if args.is_present("sections") {
        view.add(InspectItem::Sections(InspectSectionsItem::new(
            grib.sections(),
        )));
    }
    if args.is_present("submessages") {
        view.add(InspectItem::SubMessages(InspectSubMessagesItem::new(
            grib.submessages(),
        )));
    }
    if args.is_present("templates") {
        let tmpls = grib.list_templates();
        view.add(InspectItem::Templates(InspectTemplatesItem::new(tmpls)));
    }
    if view.items.len() == 0 {
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

        if let Some(_) = items.peek() {
            println!("");
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
        !(self.items.len() < 2)
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
    data: &'i Box<[SectionInfo]>,
}

impl<'i> InspectSectionsItem<'i> {
    fn new(data: &'i Box<[SectionInfo]>) -> Self {
        Self { data: data }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'i> Display for InspectSectionsItem<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (i, sect) in self.data.iter().enumerate() {
            write!(
                f,
                "{:>5} │ {:016x} - {:016x} │ Section {}\n",
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
    data: &'i Box<[SubMessageIndex]>,
}

impl<'i> InspectSubMessagesItem<'i> {
    fn new(data: &'i Box<[SubMessageIndex]>) -> Self {
        Self { data: data }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl<'i> Display for InspectSubMessagesItem<'i> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fn format_section(s: usize) -> String {
            format!("{:>5}", s.to_string())
        }

        fn format_section_optional(section: Option<usize>) -> String {
            let s = match section {
                None => "-".to_string(),
                Some(id) => id.to_string(),
            };
            format!("{:>5}", s)
        }

        let header = "   id │    S2    S3    S4    S5    S6    S7\n";
        let style = Style::new().bold();
        write!(f, "{}", style.apply_to(header))?;
        for (i, submessage) in self.data.iter().enumerate() {
            write!(
                f,
                "{:>5} │ {} {} {} {} {} {}\n",
                i,
                format_section_optional(submessage.section2),
                format_section(submessage.section3),
                format_section(submessage.section4),
                format_section(submessage.section5),
                format_section(submessage.section6),
                format_section(submessage.section7),
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
        Self { data: data }
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
                    write!(f, "{:<8} - {}\n", tmpl.to_string(), s)?;
                }
                None => {
                    write!(f, "{}\n", tmpl)?;
                }
            }
        }
        Ok(())
    }
}
