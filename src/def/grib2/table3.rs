use grib_template_derive::{CodeTable, Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_3_1_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct GridDefinitionTemplateNum(
    /// Grid definition template number (= N) (see Code table 3.1).
    pub u16,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_3_2_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct EarthShapeType(
    /// Shape of the Earth (see Code table 3.2).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
#[dump(flag)]
pub struct ResolutionAndComponentFlags(
    /// Resolution and component flags (see Flag table 3.3).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
#[dump(flag)]
pub struct ScanningMode(
    /// Scanning mode (flags - see Flag table 3.4).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
#[dump(flag)]
pub struct ProjectionCentreFlag(
    /// Projection centre flag (see Flag table 3.5).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_3_11_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct PointListInterpretation(
    /// Interpretation of list of numbers (see Code table 3.11).
    pub u8,
);
