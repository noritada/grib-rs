use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct GridDefinitionTemplateNum(
    /// Grid definition template number (= N) (see Code table 3.1).
    pub u16,
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
