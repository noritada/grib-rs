use crate::{
    DecodeError,
    helpers::{GribInt, read_as},
};

pub(crate) struct Section5Param {
    pub(crate) num_points_encoded: u32,
    pub(crate) template_num: u16,
}

impl Section5Param {
    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let num_points_encoded = read_as!(u32, buf, 0);
        let template_num = read_as!(u16, buf, 4);
        Self {
            num_points_encoded,
            template_num,
        }
    }
}

pub(crate) struct SimplePackingParam {
    pub(crate) ref_val: f32,
    pub(crate) exp: i16,
    pub(crate) dig: i16,
    pub(crate) nbit: u8,
}

impl SimplePackingParam {
    pub(crate) fn from_buf(buf: &[u8]) -> Result<Self, DecodeError> {
        let ref_val = read_as!(f32, buf, 0);
        let exp = read_as!(u16, buf, 4).as_grib_int();
        let dig = read_as!(u16, buf, 6).as_grib_int();
        let nbit = read_as!(u8, buf, 8);
        let original_field_type_value = read_as!(u8, buf, 9);

        if original_field_type_value != 0 {
            return Err(crate::DecodeError::NotSupported(
                "GRIB2 code table 5.1 (type of original field values)",
                original_field_type_value.into(),
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

pub(crate) struct SpatialDifferencingParam {
    pub(crate) order: u8,
    pub(crate) extra_desc_num_octets: u8,
}

impl SpatialDifferencingParam {
    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let order = read_as!(u8, buf, 0);
        let extra_desc_num_octets = read_as!(u8, buf, 1);
        Self {
            order,
            extra_desc_num_octets,
        }
    }
}

#[cfg(feature = "ccsds-unpack-with-libaec")]
pub(crate) struct CcsdsCompressionParam {
    pub(crate) mask: u8,
    pub(crate) block_size: u8,
    pub(crate) reference_sample_interval: u16,
}

#[cfg(feature = "ccsds-unpack-with-libaec")]
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

pub(crate) struct RunLengthPackingParam {
    pub(crate) nbit: u8,
    pub(crate) maxv: u16,
    pub(crate) max_level: u16,
    pub(crate) num_digits: u8,
}

impl RunLengthPackingParam {
    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let nbit = read_as!(u8, buf, 0);
        let maxv = read_as!(u16, buf, 1);
        let max_level = read_as!(u16, buf, 3);
        let num_digits = read_as!(u8, buf, 5);
        Self {
            nbit,
            maxv,
            max_level,
            num_digits,
        }
    }
}
