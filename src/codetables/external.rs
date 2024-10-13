use grib_codegen::parameter_codes;

use crate::Parameter;

#[parameter_codes(path = "def/wgrib2/gribtable")]
#[derive(Debug, Eq, PartialEq, Clone)]
#[repr(u32)]
/// Parameter abbreviation codes used in NCEP.
pub enum NCEPCode {}

impl TryFrom<&Parameter> for NCEPCode {
    type Error = &'static str;

    fn try_from(value: &Parameter) -> Result<Self, Self::Error> {
        Self::try_from(value.as_u32())
    }
}
