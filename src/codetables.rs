#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConversionError {
    Unimplemented(u8),
}

/// Implements "Code Table 1.0: GRIB Master Tables Version Number"
const CODE_TABLE_1_0: &'static [&'static str] = &[
    "Experimental",
    "Version implemented on 7 November 2001",
    "Version implemented on 4 November 2003",
    "Version implemented on 2 November 2005",
    "Version implemented on 7 November 2007",
];

/// Implements "Code Table 1.1: GRIB Local Tables Version Number"
const CODE_TABLE_1_1: &'static [&'static str] = &[
    "Local tables not used. Only table entries and templates from the current Master table are valid.",
];

/// Implements "Code Table 1.2: Significance of Reference Time"
const CODE_TABLE_1_2: &'static [&'static str] = &[
    "Analysis",
    "Start of forecast",
    "Verifying time of forecast",
    "Observation time",
];

/// Implements "Code Table 1.3: Production status of data"
const CODE_TABLE_1_3: &'static [&'static str] = &[
    "Operational products",
    "Operational test products",
    "Research products",
    "Re-analysis products",
];

/// Implements "Code Table 1.4: Type of data"
const CODE_TABLE_1_4: &'static [&'static str] = &[
    "Analysis products",
    "Forecast products",
    "Analysis and forecast products",
    "Control forecast products",
    "Perturbed forecast products",
    "Control and perturbed forecast products",
    "Processed satellite observations",
    "Processed radar observations",
];

pub fn lookup_table(
    table: &'static [&'static str],
    code: u8,
) -> Result<&'static &'static str, ConversionError> {
    table
        .get(code as usize)
        .ok_or(ConversionError::Unimplemented(code))
}
