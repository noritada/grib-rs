use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct LocalTableVer(
    /// Version number of GRIB Local tables used to augment Master tables (see
    /// Code table 1.1 and Note 2).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct RefTimeSignificance(
    /// Significance of reference time (see Code table 1.2).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct ProductionStatus(
    /// Production status of processed data in this GRIB message (see Code table
    /// 1.3).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct ProcessedDataType(
    /// Type of processed data in this GRIB message (see Code table 1.4).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct IdentificationTemplateNum(
    /// Identification template number (optional, see Code table 1.5).
    pub u16,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct CalendarType(
    /// Type of calendar (see Code table 1.6).
    pub u8,
);
