use super::wmo_csv::{Entry, parse_codes};

const C00_HEADER: [&str; 5] = [
    "GRIB version number",
    "BUFR version number",
    "CREX version number",
    "Effective date",
    "Status",
];
const C11_HEADER: [&str; 4] = [
    "CREX2",
    "GRIB2_BUFR4",
    "OriginatingGeneratingCentre_en",
    "Status",
];

pub(super) fn parse_c00(contents: &str, max_code: u16) -> Result<Vec<Entry>, String> {
    parse(contents, max_code, &C00_HEADER, 0, 3, false)
}

pub(super) fn parse_c11(contents: &str, max_code: u16) -> Result<Vec<Entry>, String> {
    parse(contents, max_code, &C11_HEADER, 1, 2, true)
}

fn parse(
    contents: &str,
    max_code: u16,
    header: &[&str],
    code_column: usize,
    description_column: usize,
    skip_not_applicable: bool,
) -> Result<Vec<Entry>, String> {
    let mut reader = csv::Reader::from_reader(contents.as_bytes());
    if !reader
        .headers()
        .map_err(|error| error.to_string())?
        .iter()
        .eq(header.iter().copied())
    {
        return Err(format!("expected WMO CCT CSV header: {}", header.join(",")));
    }
    let mut entries: Vec<Entry> = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|error| error.to_string())?;
        let code = &record[code_column];
        if code.is_empty() || (skip_not_applicable && code == "Not applicable") {
            continue;
        }
        let line = record.position().map_or(0, |position| position.line());
        let codes = parse_codes(code, max_code)
            .map_err(|error| format!("line {line}, {}: {error}", header[code_column]))?;
        // Validate even entries whose descriptions are suppressed.
        if entries
            .iter()
            .any(|entry| codes.start() <= entry.codes.end() && entry.codes.start() <= codes.end())
        {
            return Err(format!(
                "line {line}, {}: overlapping code range {code:?}",
                header[code_column]
            ));
        }
        // Keep the exclusions consistent with gen/src/lib.rs.
        let description = match &record[description_column] {
            ""
            | "Future versions"
            | "Reserved"
            | "Reserved for local use"
            | "Reserved for other centres"
            | "Missing"
            | "Missing value"
            | ")" => None,
            description => Some(description.to_owned()),
        };
        entries.push(Entry {
            codes,
            description,
            unit: None,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c11(rows: &[(&str, &str)]) -> Result<String, Box<dyn std::error::Error>> {
        let mut writer = csv::Writer::from_writer(Vec::new());
        writer.write_record(C11_HEADER)?;
        for (code, description) in rows {
            writer.write_record(["", code, description, "Operational"])?;
        }
        Ok(String::from_utf8(writer.into_inner()?)?)
    }

    #[test]
    fn c00_uses_grib_versions_and_effective_dates() -> Result<(), Box<dyn std::error::Error>> {
        let contents = format!(
            "{}\n0,0,0,Experimental,Operational\n,1,,BUFR only,Operational\n1,2,2,1 January 2000,Operational\n2-254,,,Future versions,Operational\n255,,,Missing,Operational\n",
            C00_HEADER.join(","),
        );
        let entries = parse_c00(&contents, 255)?;
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].description.as_deref(), Some("Experimental"));
        assert_eq!(entries[1].codes, 1..=1);
        assert_eq!(entries[1].description.as_deref(), Some("1 January 2000"));
        assert_eq!(entries[2].codes, 2..=254);
        assert_eq!(entries[2].description, None);
        assert_eq!(entries[3].description, None);
        assert!(entries.iter().all(|entry| entry.unit.is_none()));
        Ok(())
    }

    #[test]
    fn c11_skips_headings_and_non_grib_codes() -> Result<(), Box<dyn std::error::Error>> {
        let contents = c11(&[
            ("", "00001-00009: WMCs"),
            ("Not applicable", "Not used"),
            ("00001", "Example, with \"quotes\""),
            ("10-12", "Another centre"),
        ])?;
        let entries = parse_c11(&contents, 65535)?;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].codes, 1..=1);
        assert_eq!(
            entries[0].description.as_deref(),
            Some("Example, with \"quotes\"")
        );
        assert_eq!(entries[1].codes, 10..=12);
        Ok(())
    }

    #[test]
    fn suppresses_placeholder_descriptions() -> Result<(), Box<dyn std::error::Error>> {
        for description in [
            "",
            "Future versions",
            "Reserved",
            "Reserved for local use",
            "Reserved for other centres",
            "Missing",
            "Missing value",
            ")",
        ] {
            let entries = parse_c11(&c11(&[("0", description)])?, 65535)?;
            assert_eq!(entries[0].description, None);
        }
        assert_eq!(
            parse_c11(&c11(&[("255", "Not to be used")])?, 65535)?[0]
                .description
                .as_deref(),
            Some("Not to be used"),
        );
        Ok(())
    }

    macro_rules! test_invalid_codes {
        ($($name:ident: $rows:expr, $max:expr => $message:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() -> Result<(), Box<dyn std::error::Error>> {
                    let error = parse_c11(&c11($rows)?, $max).expect_err("invalid codes");
                    assert!(error.contains($message), "{error}");
                    assert!(error.contains("line "), "{error}");
                    Ok(())
                }
            )*
        };
    }

    test_invalid_codes! {
        duplicate: &[("1", "Centre"), ("1", "Other centre")], 65535 => "overlapping",
        shared_endpoint: &[("1-3", "Reserved"), ("3-5", "Reserved")], 65535 => "overlapping",
        contained: &[("1-10", "Reserved"), ("5", "Centre")], 65535 => "overlapping",
        reverse_order_overlap: &[("5", "Centre"), ("1-10", "Reserved")], 65535 => "overlapping",
        reversed: &[("3-1", "Centre")], 65535 => "reversed",
        malformed: &[("abc", "Centre")], 65535 => "invalid code",
        overflow: &[("65536", "Centre")], 65535 => "invalid code",
        exceeds_type: &[("256", "Centre")], 255 => "exceeds",
    }

    #[test]
    fn validates_headers_and_record_lengths() {
        assert!(parse_c00("wrong,header\n", 255).is_err());
        assert!(parse_c11("wrong,header\n", 65535).is_err());
        let contents = format!("{}\n0,0,0\n", C00_HEADER.join(","));
        assert!(parse_c00(&contents, 255).is_err());
    }

    #[test]
    fn adjacent_ranges_are_valid() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            parse_c11(&c11(&[("3-5", "Centre"), ("1-2", "Centre")])?, 65535)?.len(),
            2
        );
        Ok(())
    }

    #[test]
    fn c00_rejects_overlap_and_non_numeric_codes() {
        for rows in [
            "0,,,Experimental,Operational\n0-2,,,Future versions,Operational\n",
            "Not applicable,,,Not used,Operational\n",
        ] {
            assert!(parse_c00(&format!("{}\n{rows}", C00_HEADER.join(",")), 255).is_err());
        }
    }
}
