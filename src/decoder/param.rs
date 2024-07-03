use crate::helpers::{read_as, GribInt};

pub(crate) struct SimplePackingParam {
    pub(crate) ref_val: f32,
    pub(crate) exp: i16,
    pub(crate) dig: i16,
    pub(crate) nbit: u8,
}

impl SimplePackingParam {
    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let ref_val = read_as!(f32, buf, 0);
        let exp = read_as!(u16, buf, 4).as_grib_int();
        let dig = read_as!(u16, buf, 6).as_grib_int();
        let nbit = read_as!(u8, buf, 8);
        Self {
            ref_val,
            exp,
            dig,
            nbit,
        }
    }
}

pub(crate) struct ComplexPackingParam {
    pub(crate) group_splitting_method_used: u8,
    pub(crate) missing_value_management_used: u8,
    pub(crate) ngroup: u32,
    pub(crate) group_width_ref: u8,
    pub(crate) group_width_nbit: u8,
    pub(crate) group_len_ref: u32,
    pub(crate) group_len_inc: u8,
    pub(crate) group_len_last: u32,
    pub(crate) group_len_nbit: u8,
}

impl ComplexPackingParam {
    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let group_splitting_method_used = read_as!(u8, buf, 0);
        let missing_value_management_used = read_as!(u8, buf, 1);
        let ngroup = read_as!(u32, buf, 10);
        let group_width_ref = read_as!(u8, buf, 14);
        let group_width_nbit = read_as!(u8, buf, 15);
        let group_len_ref = read_as!(u32, buf, 16);
        let group_len_inc = read_as!(u8, buf, 20);
        let group_len_last = read_as!(u32, buf, 21);
        let group_len_nbit = read_as!(u8, buf, 25);
        Self {
            group_splitting_method_used,
            missing_value_management_used,
            ngroup,
            group_width_ref,
            group_width_nbit,
            group_len_ref,
            group_len_inc,
            group_len_last,
            group_len_nbit,
        }
    }
}
