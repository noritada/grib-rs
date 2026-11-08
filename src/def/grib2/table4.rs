use grib_template_derive::{CodeTable, Dump, FixedSize, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_0_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct ProductDefinitionTemplateNum(
    /// Product definition template number (see Code table 4.0).
    pub u16,
);

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable, FixedSize,
)]
#[code_table(
    file = "def/GRIB2/GRIB2_CodeFlag_4_1_CodeTable_en.csv",
    format = wmo_grib2_tables,
    context = crate::Grib2Context,
    key = (context.discipline,),
)]
#[dump(code_table, context = crate::Grib2Context)]
pub struct ParamCategory(
    /// Parameter category (see Code Table 4.1).
    pub u8,
);

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable, FixedSize,
)]
#[code_table(
    file = "def/GRIB2/GRIB2_CodeFlag_4_2_CodeTable_en.csv",
    format = wmo_grib2_tables,
    context = crate::ParameterContext,
    key = (context.discipline, context.category),
)]
#[dump(code_table, context = crate::ParameterContext)]
pub struct ParamNum(
    /// Parameter number (see Code Table 4.2).
    pub u8,
);

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable, FixedSize,
)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_3_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct GeneratingProcessType(
    /// Type of generating process (see Code Table 4.3).
    pub u8,
);

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable, FixedSize,
)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_4_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct TimeRangeUnit(
    /// Unit of time range (see Code Table 4.4).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_5_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct FixedSurfaceType(
    /// Type of fixed surface (see Code Table 4.5).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_6_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct EnsembleForecastType(
    /// Type of ensemble forecast (see Code Table 4.6).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_7_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct DerivedForecastType(
    /// Derived forecast (see Code Table 4.7).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_9_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct ProbabilityForecastType(
    /// Probability type (see Code Table 4.9).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_10_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct StatisticalProcessType(
    /// Statistical process used to calculate the processed field from the
    /// field at each time increment during the time range (see Code Table
    /// 4.10).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_4_11_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct TimeIncrementType(
    /// Type of time increment between successive fields used in the
    /// statistical processing (see Code Table 4.11).
    pub u8,
);
