use quick_xml::events::Event;
use quick_xml::Reader;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
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

fn parse_cct11(path: PathBuf) -> Vec<(String, String)> {
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

fn rebuild_cct11(input: Vec<(String, String)>) -> Vec<String> {
    let mut output = Vec::new();

    let mut count = 0;
    let mut empty_count = 0;

    macro_rules! flush {
        () => {{
            while empty_count > 0 {
                output.push(String::new());
                count += 1;
                empty_count -= 1;
            }
        }};
    }

    for entry in input {
        let (id, string) = entry;
        let string = match string.as_str() {
            ")" => None,
            "Reserved" => None,
            "Reserved for other centres" => None,
            "Missing value" => None,
            _ => Some(string),
        };

        if let Ok(number) = id.parse::<usize>() {
            if let Some(string) = string {
                flush!();

                assert_eq!(count, number);
                output.push(string);
                count += 1;
            } else {
                empty_count += 1;
            }
        } else if let Ok(range) = id.parse::<CodeRange>() {
            if let Some(string) = string {
                flush!();

                for _i in range.start..=range.end {
                    assert_eq!(count, _i);
                    output.push(string.clone());
                    count += 1;
                }
            } else {
                empty_count += range.size();
            }
        }
    }

    output
}

fn main() {
    let input_path = Path::new("def").join("CCT").join("C11.xml");
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let output_path = Path::new(&out_dir).join("cct11.rs");

    let parsed = parse_cct11(input_path);
    let built = rebuild_cct11(parsed);
    fs::write(
        &output_path,
        format!(
            "pub const COMMON_CODE_TABLE_11: &'static [&'static str] = &{:#?};",
            built
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=def/CCT/C11.xml");
}
