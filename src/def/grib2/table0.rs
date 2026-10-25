use grib_template_derive::{CodeTable, Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_0_0_CodeTable_en.csv", format = wmo_csv)]
#[dump(code_table)]
pub struct Discipline(
    /// Discipline - GRIB Master Table Number (see Code Table 0.0).
    pub u8,
);
