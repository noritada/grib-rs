use crate::utils::{read_as, GribInt};

pub(crate) struct SimplePackingParam {
    pub(crate) ref_val: f32,
    pub(crate) exp: i16,
    pub(crate) dig: i16,
    pub(crate) nbit: u8,
}

impl SimplePackingParam {
    pub(crate) fn new(ref_val: f32, exp: i16, dig: i16, nbit: u8) -> Self {
        Self {
            ref_val,
            exp,
            dig,
            nbit,
        }
    }

    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let ref_val = read_as!(f32, buf, 0);
        let exp = read_as!(u16, buf, 4).as_grib_int();
        let dig = read_as!(u16, buf, 6).as_grib_int();
        let nbit = read_as!(u8, buf, 8);
        Self::new(ref_val, exp, dig, nbit)
    }
}
