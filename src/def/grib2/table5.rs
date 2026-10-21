use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct DataRepresentationTemplateNum(
    /// Data representation template number (see Code table 5.0).
    pub u16,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
#[dump(flag)]
pub struct OrderingFlags(
    /// Ordering flags (Flag table 5.50002).
    pub u8,
);
