use grib_template_derive::{Dump, TryFromSlice};

/// Identification template 1.0 - calendar definition.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template1_0 {
    /// Type of calendar (see Code table 1.6).
    pub calendar_type: u8,
}

/// Identification template 1.1 - paleontological offset.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template1_1 {
    /// Number of tens of thousands of years of offset.
    pub paleontological_offset: u16,
}

/// Identification template 1.2 - calendar definition and paleontological
/// offset.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template1_2 {
    /// Type of calendar (see Code table 1.6).
    pub calendar_type: u8,
    /// Number of tens of thousands of years of offset.
    pub paleontological_offset: u16,
}
