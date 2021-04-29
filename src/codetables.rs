use std::fmt::{self, Display, Formatter};

pub struct LookupResult(Result<&'static &'static str, ConversionError>);

impl Display for LookupResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let s = match &self.0 {
            Ok(s) => format!("{}", s),
            Err(e) => format!("{}", e),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversionError {
    Unimplemented(usize),
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Unimplemented(code) => write!(f, "code '{}' is not implemented", code),
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/cct.rs"));
include!(concat!(env!("OUT_DIR"), "/grib2_codeflag.rs"));

/// Implements "Code Table 1.1: GRIB Local Tables Version Number"
pub const CODE_TABLE_1_1: &'static [&'static str] = &[
    "Local tables not used. Only table entries and templates from the current Master table are valid.",
];

pub fn lookup_table(table: &'static [&'static str], code: usize) -> LookupResult {
    let result = table.get(code).ok_or(ConversionError::Unimplemented(code));
    LookupResult(result)
}
