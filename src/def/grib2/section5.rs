use grib_template_derive::{Dump, TryFromSlice};

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section5Param {
    pub header: SectionHeader,
    pub payload: Section5Payload,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct SectionHeader {
    /// Length of section in octets (nn).
    pub len: u32,
    /// Number of section (5).
    pub sect_num: u8,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section5Payload {
    /// Number of data points where one or more values are specified in Section
    /// 7 when a bit map is present, total number of data points when a bit map
    /// is absent.
    pub num_encoded_points: u32,
    /// Data representation template number (see Code table 5.0).
    pub template_num: u16,
    /// Data representation template (see template 5.X, where X is the data
    /// representation template number given in octets 10–11).
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
    /// Reference value (R) (IEEE 32-bit floating-point value).
    pub ref_val: f32,
    /// Binary scale factor (E).
    pub exp: i16,
    /// Decimal scale factor (D).
    pub dec: i16,
    /// Number of bits used for each packed value for simple packing, or for
    /// each group reference value for complex packing or spatial differencing.
    pub num_bits: u8,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
}

impl SimplePackingParam {
    pub(crate) fn is_supported(&self) -> Result<(), crate::DecodeError> {
        if self.orig_field_type != 0 {
            return Err(crate::DecodeError::NotSupported(
                "GRIB2 code table 5.1 (type of original field values)",
                self.orig_field_type.into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn zero_bit_reference_value(&self) -> f32 {
        self.ref_val * 10_f32.powi(-i32::from(self.dec))
    }
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct ComplexPackingParam {
    /// Group splitting method used (see Code table 5.4).
    pub group_splitting_method: u8,
    /// Missing value management used (see Code table 5.5).
    pub missing_value_management: u8,
    /// Primary missing value substitute.
    pub primary_missing_value: u32,
    /// Secondary missing value substitute.
    pub secondary_missing_value: u32,
    /// NG - number of groups of data values into which field is split.
    pub num_groups: u32,
    /// Reference for group widths (see Note 12).
    pub group_width_ref: u8,
    /// Number of bits used for the group widths (after the reference value in
    /// octet 36 has been removed).
    pub num_group_width_bits: u8,
    /// Reference for group lengths (see Note 13).
    pub group_len_ref: u32,
    /// Length increment for the group lengths (see Note 14).
    pub group_len_inc: u8,
    /// True length of last group.
    pub group_len_last: u32,
    /// Number of bits used for the scaled group lengths (after subtraction of
    /// the reference value given in octets 38-41 and division by the length
    /// increment given in octet 42).
    pub num_group_len_bits: u8,
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct SpatialDifferencingParam {
    /// Order of spatial differencing (see Code table 5.6).
    pub order: u8,
    /// Number of octets required in the data section to specify extra
    /// descriptors needed for spatial differencing (octets 6-ww in data
    /// template 7.3).
    pub num_extra_desc_octets: u8,
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct CcsdsCompressionParam {
    /// CCSDS compression options mask (see Note 3).
    pub mask: u8,
    /// Block size.
    pub block_size: u8,
    /// Reference sample interval.
    pub ref_sample_interval: u16,
}

#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct RunLengthPackingParam {
    /// Number of bits used for each packed value in the run length packing with
    /// level value.
    pub num_bits: u8,
    /// MV - maximum value within the levels that are used in the packing.
    pub max_val: u16,
    /// MVL - maximum value of level (predefined).
    pub max_level: u16,
    /// Decimal scale factor of representative value of each level.
    pub dec: u8,
    /// List of MVL scaled representative values of each level from lv=1 to MVL.
    #[grib_template(len = "max_level")]
    pub level_vals: Vec<u16>,
}
