use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::Parameter;

#[derive(Debug, Eq, PartialEq, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
/// Parameter code used in NCEP.
pub enum NCEP {
    /// Pressure.
    PRES = 0x_00_03_00,
    /// Pressure reduced to MSL.
    PRMSL = 0x_00_03_01,
    /// Geopotential Height.
    HGT = 0x_00_03_05,
}

impl TryFrom<&Parameter> for NCEP {
    type Error = &'static str;

    fn try_from(value: &Parameter) -> Result<Self, Self::Error> {
        let code = value.as_u32();
        Self::try_from_primitive(code).map_err(|_| "code not found")
    }
}
