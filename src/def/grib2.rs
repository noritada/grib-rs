//! Definitions of parameters contained in GRIB2 data.

use grib_template_derive::{Dump, TryFromSlice};

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section<T>
where
    T: PartialEq + grib_template_helpers::TryFromSlice + grib_template_helpers::Dump,
{
    pub header: SectionHeader,
    pub payload: T,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct SectionHeader {
    /// Length of section in octets (nn).
    pub len: u32,
    /// Number of section.
    pub sect_num: u8,
}

/// Section 1 - Identification section.
pub type Section1 = Section<Section1Payload>;

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section1Payload {
    /// Identification of originating/generating centre (see Common Code table
    /// C-11).
    pub centre_id: u16,
    /// Identification of originating/generating subcentre (allocated by
    /// originating/generating centre).
    pub subcentre_id: u16,
    /// GRIB master table version number (see Common Code table C-0 and Note 1).
    pub master_table_version: u8,
    /// Version number of GRIB Local tables used to augment Master tables (see
    /// Code table 1.1 and Note 2).
    pub local_table_version: u8,
    /// Significance of reference time (see Code table 1.2).
    pub ref_time_significance: u8,
    /// Reference time of data.
    pub ref_time: RefTime,
    /// Production status of processed data in this GRIB message (see Code table
    /// 1.3).
    pub prod_status: u8,
    /// Type of processed data in this GRIB message (see Code table 1.4).
    pub data_type: u8,
    pub optional: Option<Section1PayloadOptional>,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct RefTime {
    /// Year (4 digits).
    pub year: u16,
    /// Month.
    pub month: u8,
    /// Day.
    pub day: u8,
    /// Hour.
    pub hour: u8,
    /// Minute.
    pub minute: u8,
    /// Second.
    pub second: u8,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section1PayloadOptional {
    /// Identification template number (optional, see Code table 1.5).
    pub template_num: u16,
    /// Identification template (optional, see template 1.X, where X is the
    /// identification template number given in octets 22-23).
    #[grib_template(variant = "template_num")]
    pub template: IdentificationTemplate,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
#[repr(u16)]
pub enum IdentificationTemplate {
    _1_0(template1::Template1_0) = 0,
    _1_1(template1::Template1_1) = 1,
    _1_2(template1::Template1_2) = 2,
}

/// Section 5 - Data representation section.
pub type Section5 = Section<Section5Payload>;

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Section5Payload {
    /// Number of data points where one or more values are specified in Section
    /// 7 when a bit map is present, total number of data points when a bit map
    /// is absent.
    pub num_encoded_points: u32,
    /// Data representation template number (see Code table 5.0).
    pub template_num: u16,
    /// Data representation template (see template 5.X, where X is the data
    /// representation template number given in octets 10-11).
    #[grib_template(variant = "template_num")]
    pub template: DataRepresentationTemplate,
}

#[derive(Debug, PartialEq, TryFromSlice, Dump)]
#[repr(u16)]
pub enum DataRepresentationTemplate {
    _5_0(template5::Template5_0) = 0,
    _5_1(template5::Template5_1) = 1,
    _5_2(template5::Template5_2) = 2,
    _5_3(template5::Template5_3) = 3,
    _5_4(template5::Template5_4) = 4,
    _5_40(template5::Template5_40) = 40,
    _5_41(template5::Template5_41) = 41,
    _5_42(template5::Template5_42) = 42,
    _5_50(template5::Template5_50) = 50,
    _5_51(template5::Template5_51) = 51,
    _5_53(template5::Template5_53) = 53,
    _5_61(template5::Template5_61) = 61,
    _5_200(template5::Template5_200) = 200,
}

pub mod template {
    //! GRIB2 template definitions.

    pub use super::{template1::*, template5::*};

    pub mod param_set {
        //! Definitions of parameter sets used in GRIB2 templates.

        pub use super::super::template5::param_set::*;
    }
}

mod template1;
mod template5;
