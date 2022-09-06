use once_cell::sync::Lazy;
#[cfg(unix)]
use pager::Pager;
use regex::Regex;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
#[cfg(unix)]
use which::which;

use grib::reader::SeekableGrib2Reader;
use grib::Grib2;

pub fn grib<P>(path: P) -> anyhow::Result<Grib2<SeekableGrib2Reader<BufReader<File>>>>
where
    P: AsRef<Path>,
{
    let f = File::open(&path)?;
    let f = BufReader::new(f);
    let grib = grib::from_reader(f)?;
    if grib.len() == 0 {
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

    print!("{}", view);
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
pub(crate) struct CliMessageIndex(pub(crate) grib::datatypes::MessageIndex);

impl std::str::FromStr for CliMessageIndex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| {
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
