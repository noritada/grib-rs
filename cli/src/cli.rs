use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    sync::LazyLock,
};

use grib::{Grib2, SeekableGrib2Reader};
#[cfg(unix)]
use pager::Pager;
use regex::Regex;
#[cfg(unix)]
use which::which;

pub fn grib<P>(path: P) -> anyhow::Result<Grib2<SeekableGrib2Reader<std::io::Cursor<Vec<u8>>>>>
where
    P: AsRef<Path>,
{
    let mut buf = Vec::with_capacity(4096);
    if is_dash(&path) {
        let mut stdin = std::io::stdin();
        let _size = stdin.read_to_end(&mut buf);
    } else {
        let f = File::open(path)?;
        let mut f = BufReader::new(f);
        let _size = f.read_to_end(&mut buf);
    };
    let grib = grib::from_bytes(buf)?;

    if grib.is_empty() {
        anyhow::bail!("empty GRIB2 data")
    }
    Ok(grib)
}

pub(crate) fn display_in_pager<V>(view: V)
where
    V: PredictableNumLines + std::fmt::Display,
{
    let user_attended = console::user_attended();

    let term = console::Term::stdout();
    let (height, _width) = term.size();
    if view.num_lines() > height.into() {
        start_pager();
    }

    if user_attended {
        console::set_colors_enabled(true);
    }

    print!("{view}");
}

pub(crate) trait PredictableNumLines {
    fn num_lines(&self) -> usize;
}

#[cfg(unix)]
fn start_pager() {
    if which("less").is_ok() {
        Pager::with_pager("less -R").setup();
    } else {
        Pager::new().setup();
    }
}

#[cfg(not(unix))]
fn start_pager() {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CliMessageIndex(pub(crate) grib::MessageIndex);

impl std::str::FromStr for CliMessageIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(
                r"(?x)      # insignificant whitespace mode
                ^
                ([0-9]+)    # message index
                \.         # separator
                ([0-9]+)    # submessage index
                $",
            )
            .unwrap()
        });
        let cap = RE.captures(s).ok_or_else(|| {
            anyhow::anyhow!(
                "message index must be specified as 'N.M' where N and M are both integers"
            )
        })?;
        let message_index = cap.get(1).unwrap();
        let message_index = usize::from_str(message_index.as_str()).unwrap();
        let submessage_index = cap.get(2).unwrap();
        let submessage_index = usize::from_str(submessage_index.as_str()).unwrap();
        let inner = (message_index, submessage_index);
        Ok(Self(inner))
    }
}

pub(crate) enum WriteStream {
    File(BufWriter<std::fs::File>),
    Stdout(std::io::Stdout),
}

impl WriteStream {
    pub(crate) fn new<P>(out_path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let stream = if is_dash(&out_path) {
            Self::Stdout(std::io::stdout())
        } else {
            let f = File::create(out_path)?;
            let f = BufWriter::new(f);
            Self::File(f)
        };
        Ok(stream)
    }

    pub(crate) fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        match self {
            Self::File(file) => file.write_all(buf),
            Self::Stdout(stdout) => stdout.write_all(buf),
        }
    }
}

fn is_dash<P: AsRef<Path>>(path: P) -> bool {
    matches!(path.as_ref().to_str(), Some("-"))
}

macro_rules! module_component {
    () => {
        module_path!().split("::").last().unwrap_or("")
    };
}
pub(crate) use module_component;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_parsing_message_index() -> Result<(), Box<dyn std::error::Error>> {
        let actual = "1.1".parse::<CliMessageIndex>()?;
        let expected = CliMessageIndex((1, 1));
        assert_eq!(actual, expected);
        Ok(())
    }

    macro_rules! test_message_index_parsing_failures {
        ($(($name:ident, $input:expr),)*) => ($(
            #[test]
            fn $name() {
                let result= $input.parse::<CliMessageIndex>();
                assert!(result.is_err());
            }
        )*);
    }

    test_message_index_parsing_failures! {
        (message_index_parsing_failure_due_to_wrong_separator, "1_1"),
        (message_index_parsing_failure_due_to_non_digit_message_index, "a.1"),
        (message_index_parsing_failure_due_to_non_digit_submessage_index, "1.a"),
        (message_index_parsing_failure_due_to_garbase_before_index, "_1.1"),
        (message_index_parsing_failure_due_to_garbase_after_index, "1.1_"),
    }
}
