use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

use super::table1;

/// Identification template 1.0 - calendar definition.
#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template1_0 {
    pub calendar_type: table1::CalendarType,
}

/// Identification template 1.1 - paleontological offset.
#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template1_1 {
    /// Number of tens of thousands of years of offset.
    pub paleontological_offset: u16,
}

/// Identification template 1.2 - calendar definition and paleontological
/// offset.
#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template1_2 {
    pub calendar_type: table1::CalendarType,
    /// Number of tens of thousands of years of offset.
    pub paleontological_offset: u16,
}
