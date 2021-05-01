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

pub struct CommonCodeTable00;

impl ArrayLookup for CommonCodeTable00 {
    fn data(&self) -> &'static [&'static str] {
        COMMON_CODE_TABLE_00
    }
}

pub struct CommonCodeTable11;

impl ArrayLookup for CommonCodeTable11 {
    fn data(&self) -> &'static [&'static str] {
        COMMON_CODE_TABLE_11
    }
}

pub struct CodeTable0_0;

impl ArrayLookup for CodeTable0_0 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_0_0
    }
}

pub struct CodeTable1_1;

impl ArrayLookup for CodeTable1_1 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_1_1
    }
}

pub struct CodeTable1_2;

impl ArrayLookup for CodeTable1_2 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_1_2
    }
}

pub struct CodeTable1_3;

impl ArrayLookup for CodeTable1_3 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_1_3
    }
}

pub struct CodeTable1_4;

impl ArrayLookup for CodeTable1_4 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_1_4
    }
}

pub struct CodeTable3_1;

impl ArrayLookup for CodeTable3_1 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_3_1
    }
}

pub struct CodeTable4_0;

impl ArrayLookup for CodeTable4_0 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_4_0
    }
}

pub struct CodeTable5_0;

impl ArrayLookup for CodeTable5_0 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_5_0
    }
}

pub trait Lookup {
    fn lookup(&self, code: usize) -> LookupResult;
}

pub trait ArrayLookup: Lookup {
    fn data(&self) -> &'static [&'static str];
}

impl<T: ArrayLookup> Lookup for T {
    fn lookup(&self, code: usize) -> LookupResult {
        let result = self
            .data()
            .get(code)
            .ok_or(ConversionError::Unimplemented(code));
        LookupResult(result)
    }
}
