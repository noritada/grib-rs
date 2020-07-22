use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::*;

pub fn parse(path: PathBuf) -> Vec<(String, String)> {
    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);
    let mut reader = Reader::from_reader(f);

    let mut output = Vec::new();

    let mut buf = Vec::new();
    let mut texts = Vec::new();
    let mut code = None;
    let mut date = None;

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
                    b"GRIB-version-number" => {
                        code = get_text!();
                    }
                    b"Effective-date" => {
                        date = get_text!();
                    }
                    b"C00" => {
                        let code = std::mem::replace(&mut code, None);
                        let date = std::mem::replace(&mut date, None);

                        match (code, date) {
                            (None, _) => continue,
                            (Some(_), None) => {
                                panic!("<Effective-date> is missing (position: {})", pos)
                            }
                            (Some(code), Some(date)) => {
                                output.push((code, date));
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

pub fn rebuild(input: Vec<(String, String)>) -> Vec<String> {
    let mut output = Vec::new();

    let mut count = 0;
    let mut empty_count = 0;

    for entry in input {
        let (id, string) = entry;
        let string = match string.as_str() {
            "Future versions" => None,
            "Missing" => None,
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
    fn parse_all() {
        let path = Path::new("testdata").join("cct00.xml");
        assert_eq!(
            parse(path),
            vec![
                ("0", "Experimental"),
                ("1", "1 January 2000"),
                ("2", "1 January 2001"),
                ("3", "Pre-operational to be implemented by next amendment"),
                ("4-254", "Future versions"),
                ("255", "Missing"),
            ]
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn rebuild_all() {
        let input = vec![
            ("0", "Experimental"),
            ("1", "1 January 2000"),
            ("2", "1 January 2001"),
            ("3", "Pre-operational to be implemented by next amendment"),
            ("4-254", "Future versions"),
            ("255", "Missing"),
        ]
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect::<Vec<_>>();
        assert_eq!(
            rebuild(input),
            vec![
                "Experimental",
                "1 January 2000",
                "1 January 2001",
                "Pre-operational to be implemented by next amendment",
            ]
        );
    }
}
