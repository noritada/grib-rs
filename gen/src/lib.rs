use std::str::from_utf8;
use std::str::FromStr;

pub mod cct00;
pub mod cct11;

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
