use grib_template_derive::{Dump, TryFromSlice};

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section5Param {
    pub header: SectionHeader,
    pub payload: Section5Payload,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct SectionHeader {
    pub len: u32,
    pub sect_num: u8,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section5Payload {
    pub num_points_encoded: u32,
    pub template_num: u16,
    #[grib_template(variant = "template_num")]
    pub template: Template,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
#[repr(u16)]
pub enum Template {
    Simple(SimpleTemplate) = 0,
    Complex(ComplexTemplate) = 2,
    ComplexSpatial(ComplexSpatialTemplate) = 3,
    Jpeg2000(Jpeg2000Template) = 40,
    Png(PngTemplate) = 41,
    Ccsds(CcsdsCompressionTemplate) = 42,
    RunLength(RunLengthPackingTemplate) = 200,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct SimpleTemplate {
    pub simple: SimplePackingParam,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct ComplexTemplate {
    pub simple: SimplePackingParam,
    pub complex: ComplexPackingParam,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct ComplexSpatialTemplate {
    pub simple: SimplePackingParam,
    pub complex: ComplexPackingParam,
    pub spatial: SpatialDifferencingParam,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Jpeg2000Template {
    pub simple: SimplePackingParam,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct PngTemplate {
    pub simple: SimplePackingParam,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct CcsdsCompressionTemplate {
    pub simple: SimplePackingParam,
    pub ccsds: CcsdsCompressionParam,
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct RunLengthPackingTemplate {
    pub run_length: RunLengthPackingParam,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct SimplePackingParam {
    pub ref_val: f32,
    pub exp: i16,
    pub dig: i16,
    pub nbit: u8,
    pub original_field_type_value: u8,
}

impl SimplePackingParam {
    pub(crate) fn is_supported(&self) -> Result<(), crate::DecodeError> {
        if self.original_field_type_value != 0 {
            return Err(crate::DecodeError::NotSupported(
                "GRIB2 code table 5.1 (type of original field values)",
                self.original_field_type_value.into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn zero_bit_reference_value(&self) -> f32 {
        self.ref_val * 10_f32.powi(-i32::from(self.dig))
    }
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct ComplexPackingParam {
    pub group_splitting_method_used: u8,
    pub missing_value_management_used: u8,
    pub primary_missing_value: u32,
    pub secondary_missing_value: u32,
    pub ngroup: u32,
    pub group_width_ref: u8,
    pub group_width_nbit: u8,
    pub group_len_ref: u32,
    pub group_len_inc: u8,
    pub group_len_last: u32,
    pub group_len_nbit: u8,
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct SpatialDifferencingParam {
    pub order: u8,
    pub extra_desc_num_octets: u8,
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct CcsdsCompressionParam {
    pub mask: u8,
    pub block_size: u8,
    pub reference_sample_interval: u16,
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct RunLengthPackingParam {
    pub nbit: u8,
    pub maxv: u16,
    pub max_level: u16,
    pub num_digits: u8,
    #[grib_template(len = "max_level")]
    pub leval_values: Vec<u16>,
}
