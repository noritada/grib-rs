use std::ops::RangeInclusive;

const HEADER: [&str; 9] = [
    "Title_en",
    "SubTitle_en",
    "CodeFlag",
    "Value",
    "MeaningParameterDescription_en",
    "Note_en",
    "noteIDs",
    "UnitComments_en",
    "Status",
];

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Entry {
    pub codes: RangeInclusive<u16>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub missing: bool,
}

pub(super) fn parse(contents: &str, max_code: u16) -> Result<Vec<Entry>, String> {
    let mut reader = csv::Reader::from_reader(contents.as_bytes());
    let header = reader.headers().map_err(|error| error.to_string())?;
    if !header.iter().eq(HEADER) {
        return Err(format!("expected WMO CSV header: {}", HEADER.join(",")));
    }
    reader
        .records()
        .map(|record| {
            let record = record.map_err(|error| error.to_string())?;
            let codes = parse_codes(&record[2], max_code).map_err(|error| {
                let line = record
                    .position()
                    .map(|position| position.line())
                    .unwrap_or(0);
                format!("line {line}, CodeFlag: {error}")
            })?;
            let description = match &record[4] {
                "" | "Reserved" | "Reserved for local use" | "Missing" | "Missing value" => None,
                description => Some(description.to_owned()),
            };
            let unit = match &record[7] {
                "" => None,
                unit => Some(unit.to_owned()),
            };
            Ok(Entry {
                missing: matches!(&record[4], "Missing" | "Missing value"),
                codes,
                description,
                unit,
            })
        })
        .collect()
}

pub(super) fn parse_codes(value: &str, max_code: u16) -> Result<RangeInclusive<u16>, String> {
    let parse_number = |number: &str| {
        if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("invalid code number {number:?}"));
        }
        number
            .parse::<u16>()
            .map_err(|error| format!("invalid code number {number:?}: {error}"))
    };
    let (start, end) = match value.split_once('-') {
        Some((start, end)) => (parse_number(start)?, parse_number(end)?),
        None => {
            let code = parse_number(value)?;
            (code, code)
        }
    };
    if start > end {
        return Err(format!("reversed range {value:?}"));
    }
    if end > max_code {
        return Err(format!("code {end} exceeds {max_code}"));
    }
    Ok(start..=end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_table() -> Result<(), Box<dyn std::error::Error>> {
        let entries = parse(include_str!("../../tests/tables/code_table.csv"), 255)?;
        let expected = [
            (0..=0, Some("Example, with \"quotes\""), Some("m, \"unit\"")),
            (1..=5, Some("Range description"), Some("K")),
            (6..=6, None, None),
            (7..=191, None, None),
            (192..=254, None, None),
            (255..=255, None, None),
        ]
        .map(|(codes, description, unit)| Entry {
            missing: *codes.start() == 255,
            codes,
            description: description.map(str::to_owned),
            unit: unit.map(str::to_owned),
        });
        assert_eq!(entries, expected);
        Ok(())
    }

    #[test]
    fn preserves_text_and_independent_unit() -> Result<(), Box<dyn std::error::Error>> {
        let contents = format!(
            "{}\nExample,,0,,\" Description\nwith newline \",,,Reserved,Operational\nExample,,1,,Missing,,,K,Operational\n",
            HEADER.join(","),
        );
        let entries = parse(&contents, 255)?;
        assert_eq!(
            entries[0].description.as_deref(),
            Some(" Description\nwith newline ")
        );
        assert_eq!(entries[0].unit.as_deref(), Some("Reserved"));
        assert_eq!(entries[1].description, None);
        assert_eq!(entries[1].unit.as_deref(), Some("K"));
        Ok(())
    }

    macro_rules! test_codes {
        ($($name:ident: $value:expr, $max:expr => $expected:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() -> Result<(), Box<dyn std::error::Error>> {
                    assert_eq!(parse_codes($value, $max)?, $expected);
                    Ok(())
                }
            )*
        };
    }

    test_codes! {
        zero: "0", 255 => 0..=0,
        u8_max: "255", 255 => 255..=255,
        u16_max: "65535", 65535 => 65535..=65535,
        u16_range: "256-65535", 65535 => 256..=65535,
    }

    macro_rules! test_invalid_codes {
        ($($name:ident: $value:expr, $max:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() {
                    assert!(parse_codes($value, $max).is_err());
                }
            )*
        };
    }

    test_invalid_codes! {
        empty: "", 255,
        negative: "-1", 255,
        non_numeric: "x", 255,
        missing_end: "1-", 255,
        multiple_hyphens: "1-2-3", 255,
        reversed: "5-1", 255,
        u8_overflow: "256", 255,
        u8_range_overflow: "0-256", 255,
        u16_overflow: "65536", 65535,
    }

    #[test]
    fn invalid_header() {
        assert!(parse("CodeFlag,MeaningParameterDescription_en\n0,Example\n", 255).is_err());
    }

    #[test]
    fn invalid_record_length() {
        let contents = format!("{}\nExample,,0\n", HEADER.join(","));
        assert!(parse(&contents, 255).is_err());
    }

    #[test]
    fn code_error_has_line_number() {
        let contents = format!(
            "{}\nExample,,bad,,Description,,,,Operational\n",
            HEADER.join(",")
        );
        let error = parse(&contents, 255).expect_err("invalid CodeFlag should fail");
        assert!(error.contains("line 2, CodeFlag:"));
    }
}

// WMO tables 4.1 and 4.2 encode their additional keys in SubTitle_en.
pub(super) fn parse_keys(contents: &str) -> Result<Vec<Vec<u8>>, String> {
    let mut reader = csv::Reader::from_reader(contents.as_bytes());
    let mut arity = None;
    reader
        .records()
        .map(|record| {
            let record = record.map_err(|error| error.to_string())?;
            let subtitle = &record[1];
            let parse = || -> Option<Vec<u8>> {
                let rest = subtitle.strip_prefix("Product discipline ")?;
                let (discipline, rest) = rest.split_once(" - ")?;
                let mut keys = vec![discipline.parse().ok()?];
                if let Some((_, category)) = rest.split_once(", parameter category ") {
                    let (category, _) = category.split_once(':')?;
                    keys.push(category.parse().ok()?);
                }
                Some(keys)
            };
            let keys = parse().ok_or_else(|| {
                format!(
                    "line {}, invalid SubTitle_en: {subtitle:?}",
                    record.position().map_or(0, |position| position.line()),
                )
            })?;
            if let Some(arity) = arity {
                if arity != keys.len() {
                    return Err("inconsistent subtitle key count".into());
                }
            } else {
                arity = Some(keys.len());
            }
            Ok(keys)
        })
        .collect()
}

#[cfg(test)]
mod key_tests {
    use super::*;

    fn table(subtitles: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.write_record(HEADER)?;
        for subtitle in subtitles {
            writer.write_record([
                "Parameters",
                subtitle,
                "0",
                "",
                "Value",
                "",
                "",
                "",
                "Operational",
            ])?;
        }
        Ok(String::from_utf8(writer.into_inner()?)?)
    }

    #[test]
    fn subtitle_keys() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            parse_keys(&table(&["Product discipline 10 - Oceanographic products"])?)?,
            vec![vec![10]],
        );
        assert_eq!(
            parse_keys(&table(&[
                "Product discipline 0 - Meteorological products, parameter category 1: moisture"
            ])?)?,
            vec![vec![0, 1]],
        );
        Ok(())
    }

    #[test]
    fn invalid_subtitle_keys() -> Result<(), Box<dyn std::error::Error>> {
        for subtitle in [
            "",
            "Product discipline x - Invalid",
            "Product discipline 256 - Invalid",
            "Product discipline 0 - Products, parameter category x: invalid",
            "Product discipline 0 - Products, parameter category 256: invalid",
            "Product discipline 0 - Products, parameter category 0",
        ] {
            let error = parse_keys(&table(&[subtitle])?).expect_err("invalid subtitle");
            assert!(error.contains("line 2"));
        }
        assert!(
            parse_keys(&table(&[
                "Product discipline 0 - Products",
                "Product discipline 0 - Products, parameter category 0: temperature",
            ])?)
            .is_err()
        );
        Ok(())
    }
}
