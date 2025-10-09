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
    /// Number of section (5).
    pub sect_num: u8,
}

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
    /// representation template number given in octets 10–11).
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

    pub use super::template5::*;
}

mod template5;
