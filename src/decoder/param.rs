use crate::{
    helpers::{read_as, GribInt},
    GribError,
};

pub(crate) struct SimplePackingParam {
    pub(crate) ref_val: f32,
    pub(crate) exp: i16,
    pub(crate) dig: i16,
    pub(crate) nbit: u8,
}

impl SimplePackingParam {
    pub(crate) fn from_buf(buf: &[u8]) -> Result<Self, GribError> {
        let ref_val = read_as!(f32, buf, 0);
        let exp = read_as!(u16, buf, 4).as_grib_int();
        let dig = read_as!(u16, buf, 6).as_grib_int();
        let nbit = read_as!(u8, buf, 8);
        let original_field_type_value = read_as!(u8, buf, 9);

        if original_field_type_value != 0 {
            return Err(GribError::DecodeError(
                crate::DecodeError::SimplePackingDecodeError(
                    super::simple::SimplePackingDecodeError::OriginalFieldValueTypeNotSupported,
                ),
            ));
        }

        Ok(Self {
            ref_val,
            exp,
            dig,
            nbit,
        })
    }

    pub(crate) fn zero_bit_reference_value(&self) -> f32 {
        self.ref_val * 10_f32.powi(-i32::from(self.dig))
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

pub(crate) struct CcsdsCompressionParam {
    pub(crate) mask: u8,
    pub(crate) block_size: u8,
    pub(crate) reference_sample_interval: u16,
}

impl CcsdsCompressionParam {
    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let mask = read_as!(u8, buf, 0);
        let block_size = read_as!(u8, buf, 1);
        let reference_sample_interval = read_as!(u16, buf, 2);
        Self {
            mask,
            block_size,
            reference_sample_interval,
        }
    }
}
