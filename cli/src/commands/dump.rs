use std::{
    io::{BufRead, Write as _},
    path::PathBuf,
    sync::LazyLock,
};

use anyhow::Result;
use clap::{ArgMatches, Command, arg};
use console::Style;
use regex::Regex;

use crate::cli;

pub fn cli() -> Command {
    Command::new(crate::cli::module_component!())
        .about("Dump the content of GRIB submessages")
        .arg(
            arg!(<FILE> "Target file name (or a single dash (`-`) for standard input)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            arg!([INDEX] "Submessage index (all submessages if omitted)")
                .value_parser(clap::value_parser!(cli::CliMessageIndex)),
        )
        .arg(arg!(--"no-color" "Output without colorizing").action(clap::ArgAction::SetTrue))
}

pub fn exec(args: &ArgMatches) -> Result<()> {
    let file_name = args.get_one::<PathBuf>("FILE").unwrap();
    let grib = cli::grib(file_name)?;
    let mut stream = std::io::stdout();
    let mut dump = |submessage: grib::SubMessage<'_, _>| -> Result<()> {
        if args.get_flag("no-color") {
            submessage.dump(&mut stream)?;
        } else {
            // TODO: Implement multithreading for the coloring process.
            // Since the current `SubMessage` is not thread-safe and cannot be
            // used in a multithreaded environment, we will first write
            // everything to a buffer and then apply the coloring process to
            // it.

            let mut buf = std::io::Cursor::new(Vec::with_capacity(4096));
            submessage.dump(&mut buf)?;
            buf.set_position(0);
            for line in buf.lines() {
                stream.write_all(colorize(&line?).as_bytes())?;
            }
        }
        Ok(())
    };
    if let Some(cli::CliMessageIndex(message_index)) = args.get_one("INDEX") {
        let (_, submessage) = grib
            .iter()
            .find(|(index, _)| index == message_index)
            .ok_or_else(|| {
                anyhow::anyhow!("no such index: {}.{}", message_index.0, message_index.1)
            })?;
        // Display using the pager without checking the number of lines. It is
        // almost always assumed that the dump output will be longer than the
        // number of lines in the terminal.
        cli::start_pager();
        dump(submessage)?;
    } else {
        cli::start_pager();
        for (_, submessage) in grib.iter() {
            dump(submessage)?;
        }
    }
    Ok(())
}

fn colorize(line: &str) -> String {
    if line.starts_with("#") {
        let yellow = Style::new().yellow().bold();
        format!("{}\n", yellow.apply_to(line))
    } else if line.starts_with("error:") {
        let red = Style::new().red();
        format!("{}\n", red.apply_to(line))
    } else if let Some(cap) = parse_line(line) {
        format!(
            "{}{}{}{}{}{}\n",
            Style::new().green().apply_to(&cap["pos"]),
            Style::new().bold().apply_to(&cap["param"]),
            Style::new().cyan().apply_to(&cap["equal"]),
            Style::new().magenta().bold().apply_to(&cap["val"]),
            Style::new().blue().bold().apply_to(&cap["code"]),
            Style::new().dim().apply_to(&cap["comment"]),
        )
    } else {
        format!("{}\n", line)
    }
}

fn parse_line(line: &str) -> Option<regex::Captures<'_>> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?x)                    # insignificant whitespace mode
        ^
        (?<pos>[0-9]+(-[0-9]+)?)  # octet position
        (?<param>\s+\S+\s+)       # parameter
        (?<equal>=)
        (?<val>\s+.+?)  　　　       # value
        (?<code>(\s+\[.+\])?)    # code description
        (?<comment>(\s+//.+)?)    # description
        $",
        )
        .unwrap()
    });
    RE.captures(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_parser() {
        fn parse(line: &str) -> Option<[String; 6]> {
            parse_line(line).map(|cap| {
                [
                    &cap["pos"],
                    &cap["param"],
                    &cap["equal"],
                    &cap["val"],
                    &cap["code"],
                    &cap["comment"],
                ]
                .map(|s| s.to_owned())
            })
        }

        assert_eq!(
            parse("12-34     foo.bar_bar.baz = 0 [Code Zero]  // Parameter descriptions."),
            Some(
                [
                    "12-34",
                    "     foo.bar_bar.baz ",
                    "=",
                    " 0",
                    " [Code Zero]",
                    "  // Parameter descriptions."
                ]
                .map(|s| s.to_owned())
            ),
        );
        assert_eq!(
            parse("12-34     foo.bar_bar.baz = [1, 2, 3]  // Parameter descriptions."),
            Some(
                [
                    "12-34",
                    "     foo.bar_bar.baz ",
                    "=",
                    " [1, 2, 3]",
                    "",
                    "  // Parameter descriptions."
                ]
                .map(|s| s.to_owned())
            ),
        );
        assert_eq!(
            parse("12-34     foo.bar_bar.baz = [1, 2, 3]"),
            Some(
                ["12-34", "     foo.bar_bar.baz ", "=", " [1, 2, 3]", "", ""].map(|s| s.to_owned())
            ),
        );
    }
}
