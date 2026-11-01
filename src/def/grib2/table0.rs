use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct Discipline(
    /// Discipline - GRIB Master Table Number (see Code Table 0.0).
    pub u8,
);
