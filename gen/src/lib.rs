use std::str::from_utf8;
use std::str::FromStr;

pub mod cct_csv;
pub mod grib2_codeflag_csv;

pub struct CodeRange {
    start: usize,
    end: usize,
}

impl CodeRange {
    pub fn size(&self) -> usize {
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

pub enum CodeRangeParseError {
    NumberNotFound,
    HyphenNotFound,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeTable {
    desc: String,
    data: Vec<(String, String)>,
}

impl CodeTable {
    fn new(desc: String) -> Self {
        Self {
            desc: desc,
            data: Vec::new(),
        }
    }

    fn export(&self, name: &str) -> String {
        format!(
            "\
/// {}
pub const {}: &'static [&'static str] = &{:#?};",
            self.desc,
            name,
            self.to_vec(),
        )
    }

    fn to_vec(&self) -> Vec<String> {
        let mut output = Vec::new();

        let mut count = 0;
        let mut empty_count = 0;

        for entry in self.data.iter() {
            let (id, string) = entry;
            let string = match string.as_str() {
                "Future versions" => None,
                "Reserved" => None,
                "Reserved for local use" => None,
                "Reserved for other centres" => None,
                "Missing" => None,
                "Missing value" => None,
                ")" => None,
                _ => Some(string),
            };

            if let Ok(range) = id.parse::<CodeRange>() {
                if let Some(string) = string {
                    while empty_count > 0 {
                        output.push(String::new());
                        count += 1;
                        empty_count -= 1;
                    }

                    if count != range.start {
                        return Vec::new(); // Sparse code tables are not supported at the moment.
                    }
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
