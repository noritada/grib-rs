use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::from_utf8;
use std::str::FromStr;

struct CodeRange {
    start: usize,
    end: usize,
}

impl CodeRange {
    fn size(&self) -> usize {
        self.end - self.start + 1
    }
}

impl FromStr for CodeRange {
    type Err = CodeRangeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.as_bytes();
        let pos = 0;

        fn read_number(
            input: &[u8],
            mut pos: usize,
        ) -> Result<(usize, usize), CodeRangeParseError> {
            let start = pos;
            while pos < input.len() && (b'0'..=b'9').contains(&input[pos]) {
                pos += 1;
            }
            let number = from_utf8(&input[start..pos])
                .unwrap()
                .parse::<usize>()
                .or(Err(CodeRangeParseError::NumberNotFound))?;
            Ok((number, pos))
        }

        fn read_hyphen(input: &[u8], pos: usize) -> Result<usize, CodeRangeParseError> {
            if input[pos] == b'-' {
                Ok(pos + 1)
            } else {
                Err(CodeRangeParseError::HyphenNotFound)
            }
        }

        let (start, pos) = read_number(input, pos)?;
        if pos == input.len() {
            return Ok(CodeRange {
                start: start,
                end: start,
            });
        }

        let pos = read_hyphen(input, pos)?;
        let (end, _pos) = read_number(input, pos)?;

        Ok(CodeRange {
            start: start,
            end: end,
        })
    }
}

enum CodeRangeParseError {
    NumberNotFound,
    HyphenNotFound,
}

pub fn parse_cct11(path: PathBuf) -> Vec<(String, String)> {
    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);
    let mut reader = Reader::from_reader(f);

    let mut output = Vec::new();

    let mut buf = Vec::new();
    let mut texts = Vec::new();
    let mut code = None;
    let mut center = None;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(_)) => {
                texts.clear();
            }
            Ok(Event::Text(e)) => {
                texts.push(e.unescape_and_decode(&reader).unwrap());
            }
            Ok(Event::End(ref e)) => {
                let pos = reader.buffer_position();

                macro_rules! get_text {
                    () => {{
                        match texts.len() {
                            0 => panic!("Element has no text (position: {})", pos),
                            1 => Some(texts.pop().unwrap()),
                            _ => Some(texts.join("")),
                        }
                    }};
                }

                match e.name() {
                    b"GRIB2_BUFR4" => {
                        code = get_text!();
                    }
                    b"OriginatingGeneratingCentre_en" => {
                        center = get_text!();
                    }
                    b"C11" => {
                        let code = std::mem::replace(&mut code, None);
                        let center = std::mem::replace(&mut center, None);

                        match (code, center) {
                            (None, _) => continue,
                            (Some(_), None) => panic!(
                                "<OriginatingGeneratingCentre_en> is missing (position: {})",
                                pos
                            ),
                            (Some(code), Some(center)) => {
                                output.push((code, center));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            _ => (),
        }

        buf.clear();
    }

    output
}

pub fn rebuild_cct11(input: Vec<(String, String)>) -> Vec<String> {
    let mut output = Vec::new();

    let mut count = 0;
    let mut empty_count = 0;

    for entry in input {
        let (id, string) = entry;
        let string = match string.as_str() {
            ")" => None,
            "Reserved" => None,
            "Reserved for other centres" => None,
            "Missing value" => None,
            _ => Some(string),
        };

        if let Ok(range) = id.parse::<CodeRange>() {
            if let Some(string) = string {
                while empty_count > 0 {
                    output.push(String::new());
                    count += 1;
                    empty_count -= 1;
                }

                assert_eq!(count, range.start);
                if range.size() == 1 {
                    output.push(string);
                } else {
                    for _i in range.start..=range.end {
                        output.push(string.clone());
                    }
                }
                count += range.size();
            } else {
                empty_count += range.size();
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    #[test]
    fn parse_cct11_all() {
        let path = Path::new("testdata").join("cct.xml");
        assert_eq!(
            parse_cct11(path),
            vec![
                ("0", "A"),
                ("1", "B"),
                ("2", ")"),
                ("3", "C"),
                ("4", "Reserved"),
                ("5", "D"),
                ("6", "Reserved for other centres"),
                ("7-9", "E"),
                ("10-14", "Reserved"),
                ("15", "F"),
                ("16-65534", "Reserved for other centres"),
                ("65535", "Missing value"),
                ("Not applicable", "Not used"),
            ]
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn rebuild_cct11_all() {
        let input = vec![
            ("0", "A"),
            ("1", "B"),
            ("2", ")"),
            ("3", "C"),
            ("4", "Reserved"),
            ("5", "D"),
            ("6", "Reserved for other centres"),
            ("7-9", "E"),
            ("10-14", "Reserved"),
            ("15", "F"),
            ("16-65534", "Reserved for other centres"),
            ("65535", "Missing value"),
            ("Not applicable", "Not used"),
        ]
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect::<Vec<_>>();
        assert_eq!(
            rebuild_cct11(input),
            vec!["A", "B", "", "C", "", "D", "", "E", "E", "E", "", "", "", "", "", "F",]
        );
    }
}
