use grib_template_derive::{CodeTable, Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump, CodeTable)]
#[code_table(file = "def/GRIB2/GRIB2_CodeFlag_6_0_CodeTable_en.csv", format = wmo_grib2_tables)]
#[dump(code_table)]
pub struct BitMapIndicator(
    /// Bit-map indicator (see Code table 6.0 and the Note).
    pub u8,
);
