use grib_template_derive::{CodeTable, Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/CCT/C11.csv", format = wmo_cct_c11)]
#[dump(code_table)]
pub struct CentreId(
    /// Identification of originating/generating centre (see Common Code table
    /// C-11).
    pub u16,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct SubcentreId(
    /// Identification of originating/generating subcentre (allocated by
    /// originating/generating centre).
    pub u16,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/CCT/C00.csv", format = wmo_cct_c00)]
#[dump(code_table)]
pub struct MasterTableVer(
    /// GRIB master table version number (see Common Code table C-0 and Note 1).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_1_1_CodeTable_en.csv", format = wmo_csv)]
#[dump(code_table)]
pub struct LocalTableVer(
    /// Version number of GRIB Local tables used to augment Master tables (see
    /// Code table 1.1 and Note 2).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_1_2_CodeTable_en.csv", format = wmo_csv)]
#[dump(code_table)]
pub struct RefTimeSignificance(
    /// Significance of reference time (see Code table 1.2).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_1_3_CodeTable_en.csv", format = wmo_csv)]
#[dump(code_table)]
pub struct ProductionStatus(
    /// Production status of processed data in this GRIB message (see Code table
    /// 1.3).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_1_4_CodeTable_en.csv", format = wmo_csv)]
#[dump(code_table)]
pub struct ProcessedDataType(
    /// Type of processed data in this GRIB message (see Code table 1.4).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_1_5_CodeTable_en.csv", format = wmo_csv)]
#[dump(code_table)]
pub struct IdentificationTemplateNum(
    /// Identification template number (optional, see Code table 1.5).
    pub u16,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_1_6_CodeTable_en.csv", format = wmo_csv)]
#[dump(code_table)]
pub struct CalendarType(
    /// Type of calendar (see Code table 1.6).
    pub u8,
);
