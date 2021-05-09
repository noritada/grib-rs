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

pub struct CodeTable4_1 {
    discipline: u8,
}

impl CodeTable4_1 {
    pub fn new(discipline: u8) -> Self {
        Self { discipline }
    }
}

impl ArrayLookup for CodeTable4_1 {
    fn data(&self) -> &'static [&'static str] {
        match self.discipline {
            0 => CODE_TABLE_4_1_0,
            1 => CODE_TABLE_4_1_1,
            2 => CODE_TABLE_4_1_2,
            3 => CODE_TABLE_4_1_3,
            4 => CODE_TABLE_4_1_4,
            10 => CODE_TABLE_4_1_10,
            20 => CODE_TABLE_4_1_20,
            _ => CODE_TABLE_UNSUPPORTED,
        }
    }
}

pub struct CodeTable4_2 {
    discipline: u8,
    parameter: u8,
}

impl CodeTable4_2 {
    pub fn new(discipline: u8, parameter: u8) -> Self {
        Self {
            discipline,
            parameter,
        }
    }
}

impl ArrayLookup for CodeTable4_2 {
    fn data(&self) -> &'static [&'static str] {
        match (self.discipline, self.parameter) {
            (0, 0) => CODE_TABLE_4_2_0_0,
            (0, 1) => CODE_TABLE_4_2_0_1,
            (0, 2) => CODE_TABLE_4_2_0_2,
            (0, 3) => CODE_TABLE_4_2_0_3,
            (0, 4) => CODE_TABLE_4_2_0_4,
            (0, 5) => CODE_TABLE_4_2_0_5,
            (0, 6) => CODE_TABLE_4_2_0_6,
            (0, 7) => CODE_TABLE_4_2_0_7,
            (0, 13) => CODE_TABLE_4_2_0_13,
            (0, 14) => CODE_TABLE_4_2_0_14,
            (0, 15) => CODE_TABLE_4_2_0_15,
            (0, 16) => CODE_TABLE_4_2_0_16,
            (0, 17) => CODE_TABLE_4_2_0_17,
            (0, 18) => CODE_TABLE_4_2_0_18,
            (0, 19) => CODE_TABLE_4_2_0_19,
            (0, 20) => CODE_TABLE_4_2_0_20,
            (0, 190) => CODE_TABLE_4_2_0_190,
            (0, 191) => CODE_TABLE_4_2_0_191,
            (1, 0) => CODE_TABLE_4_2_1_0,
            (1, 1) => CODE_TABLE_4_2_1_1,
            (1, 2) => CODE_TABLE_4_2_1_2,
            (2, 0) => CODE_TABLE_4_2_2_0,
            (2, 3) => CODE_TABLE_4_2_2_3,
            (2, 4) => CODE_TABLE_4_2_2_4,
            (2, 5) => CODE_TABLE_4_2_2_5,
            (3, 0) => CODE_TABLE_4_2_3_0,
            (3, 1) => CODE_TABLE_4_2_3_1,
            (3, 2) => CODE_TABLE_4_2_3_2,
            (3, 3) => CODE_TABLE_4_2_3_3,
            (3, 4) => CODE_TABLE_4_2_3_4,
            (3, 5) => CODE_TABLE_4_2_3_5,
            (3, 6) => CODE_TABLE_4_2_3_6,
            (4, 0) => CODE_TABLE_4_2_4_0,
            (4, 1) => CODE_TABLE_4_2_4_1,
            (4, 2) => CODE_TABLE_4_2_4_2,
            (4, 3) => CODE_TABLE_4_2_4_3,
            (4, 4) => CODE_TABLE_4_2_4_4,
            (4, 5) => CODE_TABLE_4_2_4_5,
            (4, 6) => CODE_TABLE_4_2_4_6,
            (4, 7) => CODE_TABLE_4_2_4_7,
            (4, 8) => CODE_TABLE_4_2_4_8,
            (4, 9) => CODE_TABLE_4_2_4_9,
            (4, 10) => CODE_TABLE_4_2_4_10,
            (10, 0) => CODE_TABLE_4_2_10_0,
            (10, 1) => CODE_TABLE_4_2_10_1,
            (10, 2) => CODE_TABLE_4_2_10_2,
            (10, 3) => CODE_TABLE_4_2_10_3,
            (10, 4) => CODE_TABLE_4_2_10_4,
            (10, 191) => CODE_TABLE_4_2_10_191,
            (20, 0) => CODE_TABLE_4_2_20_0,
            (20, 1) => CODE_TABLE_4_2_20_1,
            (20, 2) => CODE_TABLE_4_2_20_2,
            _ => CODE_TABLE_UNSUPPORTED,
        }
    }
}

pub struct CodeTable4_3;

impl ArrayLookup for CodeTable4_3 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_4_3
    }
}

pub struct CodeTable4_4;

impl ArrayLookup for CodeTable4_4 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_4_4
    }
}

pub struct CodeTable4_5;

impl ArrayLookup for CodeTable4_5 {
    fn data(&self) -> &'static [&'static str] {
        CODE_TABLE_4_5
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

const CODE_TABLE_UNSUPPORTED: &'static [&'static str] = &[];

pub(crate) const SUPPORTED_PROD_DEF_TEMPLATE_NUMBERS: [u16; 71] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 20, 30, 31, 32, 33, 34, 35, 40, 41, 42,
    43, 44, 45, 46, 47, 48, 49, 51, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 67, 68, 70, 71, 72,
    73, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 91, 254, 1000, 1001, 1002, 1100, 1101,
];
