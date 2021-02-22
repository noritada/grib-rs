use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::*;

pub struct CodeDB {
    data: Vec<(String, String, String)>,
}

impl CodeDB {
    pub fn load(path: PathBuf) -> Self {
        Self {
            data: Self::parse_file(path),
        }
    }

    pub fn parse_file(path: PathBuf) -> Vec<(String, String, String)> {
        let f = File::open(&path).unwrap();
        let f = BufReader::new(f);
        let mut reader = Reader::from_reader(f);

        let mut output = Vec::new();

        let mut buf = Vec::new();
        let mut texts = Vec::new();
        let mut title = None;
        let mut code = None;
        let mut desc = None;

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
                        b"Title_en" => {
                            title = get_text!();
                        }
                        b"CodeFlag" => {
                            code = get_text!();
                        }
                        b"MeaningParameterDescription_en" => {
                            desc = get_text!();
                        }
                        b"GRIB2_CodeFlag_en" => {
                            let title = std::mem::replace(&mut title, None);
                            let code = std::mem::replace(&mut code, None);
                            let desc = std::mem::replace(&mut desc, None);

                            if let Some(title) = title {
                                let table_id = title.split(" - ").next();
                                match (table_id, code, desc) {
                                    (None, _, _) => {
                                        panic!("<Title> is missing (position: {})", pos)
                                    }
                                    (_, None, _) => {
                                        continue;
                                    }
                                    (_, _, None) => {
                                        panic!(
                                    "<MeaningParameterDescription_en> is missing (position: {})",
                                    pos
                                )
                                    }
                                    (Some(table_id), Some(code), Some(desc)) => {
                                        output.push((table_id.to_owned(), code, desc));
                                    }
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

    pub fn export(&self, name: &str) -> Vec<String> {
        let target_table_id = format!("Code table {}", name);
        let mut output = Vec::new();

        let mut count = 0;
        let mut empty_count = 0;

        for entry in self.data.iter() {
            let (table_id, id, string) = entry;
            if table_id.as_str() != target_table_id {
                continue;
            }

            let string = match string.as_str() {
                "Reserved" => None,
                "Reserved for local use" => None,
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
                        output.push(string.to_string());
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    #[test]
    fn parse_file() {
        let path = Path::new("testdata").join("grib2_codeflag.xml");
        assert_eq!(
            CodeDB::parse_file(path),
            vec![
                ("Code table 0.0", "0", "0A"),
                ("Code table 0.0", "1", "0B"),
                ("Code table 0.0", "2-254", "Reserved"),
                ("Code table 0.0", "255", "Missing"),
                ("Code table 1.0", "0", "1A"),
                ("Code table 1.0", "1", "1B"),
                ("Code table 1.0", "2-191", "Reserved"),
                ("Code table 1.0", "192-254", "Reserved for local use"),
                ("Code table 1.0", "255", "Missing"),
            ]
            .iter()
            .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn export() {
        let path = Path::new("testdata").join("grib2_codeflag.xml");
        let db = CodeDB::load(path);
        assert_eq!(db.export("1.0"), vec!["1A", "1B",]);
    }
}
