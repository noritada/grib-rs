//! Definitions of parameters contained in GRIB2 data.
//!
//! For parameter values represented as unsigned or signed integers,
//! [`MissingValue`](crate::MissingValue) trait can be used to check whether
//! they are missing values.

use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Clone, TryFromSlice, Dump)]
pub struct Section<T>
where
    T: PartialEq + crate::TryFromSlice + crate::Dump,
{
    pub header: SectionHeader,
    pub payload: T,
}

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct SectionHeader {
    /// Length of section in octets (nn).
    pub len: u32,
    /// Number of section.
    pub sect_num: u8,
}

/// SECTION 0 - Indicator section.
#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Section0 {
    /// "GRIB" (coded according to the International Alphabet No. 5).
    pub identifier: [u8; 4],
    /// Reserved.
    pub reserved: [u8; 2],
    pub discipline: table0::Discipline,
    /// GRIB Edition Number (currently 2).
    pub edition_num: u8,
    /// Total length of GRIB message in octets (including Section 0).
    pub total_len: u64,
}

/// Section 1 - Identification section.
pub type Section1 = Section<Section1Payload>;

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Section1Payload {
    /// Identification of originating/generating centre (see Common Code table
    /// C-11).
    pub centre_id: u16,
    /// Identification of originating/generating subcentre (allocated by
    /// originating/generating centre).
    pub subcentre_id: u16,
    /// GRIB master table version number (see Common Code table C-0 and Note 1).
    pub master_table_version: u8,
    pub local_table_version: table1::LocalTableVer,
    pub ref_time_significance: table1::RefTimeSignificance,
    /// Reference time of data.
    #[dump(doc(
        year = "Year (4 digits) - Reference time of data.",
        month = "Month - Reference time of data.",
        day = "Day - Reference time of data.",
        hour = "Hour - Reference time of data.",
        minute = "Minute - Reference time of data.",
        second = "Second - Reference time of data.",
    ))]
    pub ref_time: template::param_set::DateTime,
    pub prod_status: table1::ProductionStatus,
    pub data_type: table1::ProcessedDataType,
    pub optional: Option<Section1PayloadOptional>,
}

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Section1PayloadOptional {
    pub template_num: table1::IdentificationTemplateNum,
    /// Identification template (optional, see template 1.X, where X is the
    /// identification template number given in octets 22-23).
    #[grib_template(variant = "template_num")]
    pub template: IdentificationTemplate,
}

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
#[non_exhaustive]
#[repr(u16)]
pub enum IdentificationTemplate {
    _1_0(template1::Template1_0) = 0,
    _1_1(template1::Template1_1) = 1,
    _1_2(template1::Template1_2) = 2,
}

/// Section 3 - Grid definition section.
pub type Section3 = Section<Section3Payload>;

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Section3Payload {
    /// Source of grid definition (see Code table 3.0 and Note 1).
    pub grid_def_source: u8,
    /// Number of data points.
    pub num_points: u32,
    /// Number of octets for optional list of numbers (see Note 2).
    pub num_point_list_octets: u8,
    pub point_list_interpretation: table3::PointListInterpretation,
    pub template_num: table3::GridDefinitionTemplateNum,
    /// Grid Definition Template (see Template 3.N, where N is the Grid
    /// Definition Template Number given in octets 13-14).
    #[grib_template(variant = "template_num")]
    pub template: GridDefinitionTemplate,
}

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
#[non_exhaustive]
#[repr(u16)]
pub enum GridDefinitionTemplate {
    _3_0(template3::Template3_0) = 0,
    _3_1(template3::Template3_1) = 1,
    _3_2(template3::Template3_2) = 2,
    _3_3(template3::Template3_3) = 3,
    _3_10(template3::Template3_10) = 10,
    _3_20(template3::Template3_20) = 20,
    _3_30(template3::Template3_30) = 30,
    _3_40(template3::Template3_40) = 40,
    _3_41(template3::Template3_41) = 41,
    _3_42(template3::Template3_42) = 42,
    _3_43(template3::Template3_43) = 43,
    _3_101(template3::Template3_101) = 101,
}

/// Section 4 - Product definition section.
pub type Section4 = Section<Section4Payload>;

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Section4Payload {
    /// Number of coordinate values after template or number of information
    /// according to 3D vertical coordinate GRIB2 message (see Notes 1 and 5).
    pub num_coord_values: u16,
    pub template_num: table4::ProductDefinitionTemplateNum,
    /// Product Definition Template (see Template 4.X, where X is the Product
    /// Definition Template Number given in octets 8-9).
    #[grib_template(variant = "template_num")]
    pub template: ProductDefinitionTemplate,
}

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
#[non_exhaustive]
#[repr(u16)]
pub enum ProductDefinitionTemplate {
    _4_0(template4::Template4_0) = 0,
    _4_1(template4::Template4_1) = 1,
    _4_2(template4::Template4_2) = 2,
    _4_5(template4::Template4_5) = 5,
    _4_6(template4::Template4_6) = 6,
    _4_8(template4::Template4_8) = 8,
    _4_9(template4::Template4_9) = 9,
    _4_10(template4::Template4_10) = 10,
    _4_11(template4::Template4_11) = 11,
    _4_12(template4::Template4_12) = 12,
}

/// Section 5 - Data representation section.
pub type Section5 = Section<Section5Payload>;

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
pub struct Section5Payload {
    /// Number of data points where one or more values are specified in Section
    /// 7 when a bit map is present, total number of data points when a bit map
    /// is absent.
    pub num_encoded_points: u32,
    pub template_num: table5::DataRepresentationTemplateNum,
    /// Data representation template (see template 5.X, where X is the data
    /// representation template number given in octets 10-11).
    #[grib_template(variant = "template_num")]
    pub template: DataRepresentationTemplate,
}

#[derive(Debug, PartialEq, Clone, TryFromSlice, WriteToBuffer, Dump)]
#[non_exhaustive]
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
    _5_50002(template5::Template5_50002) = 50002,
}

/// Section 6 - Bit-map section.
pub type Section6 = Section<Section6Payload>;

#[derive(Debug, PartialEq, Clone, TryFromSlice, Dump)]
pub struct Section6Payload {
    /// Bit-map indicator (see Code table 6.0 and the Note).
    pub bitmap_indicator: u8,
}

pub mod table {
    pub use super::{table0::*, table1::*, table3::*, table4::*, table5::*};
}

pub mod template {
    //! GRIB2 template definitions.

    pub use super::{template1::*, template3::*, template4::*, template5::*};

    pub mod param_set {
        //! Definitions of parameter sets used in GRIB2 templates.

        use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

        use super::super::table4;
        pub use super::super::{
            template3::param_set::*, template4::param_set::*, template5::param_set::*,
        };

        /// Date and time.
        #[derive(Debug, PartialEq, Eq, Clone, TryFromSlice, WriteToBuffer, Dump)]
        pub struct DateTime {
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

        /// Value with a scaling.
        #[derive(Debug, PartialEq, Eq, Clone, TryFromSlice, WriteToBuffer, Dump)]
        pub struct ScaledValue<F, V>
        where
            F: crate::TryFromSlice + crate::WriteToBuffer + grib_template_helpers::DumpField,
            V: crate::TryFromSlice + crate::WriteToBuffer + grib_template_helpers::DumpField,
        {
            /// Scale factor.
            pub scale_factor: F,
            /// Scaled value.
            pub scaled_value: V,
        }

        /// Time range.
        #[derive(Debug, PartialEq, Eq, Clone, TryFromSlice, WriteToBuffer, Dump)]
        pub struct TimeRange<L>
        where
            L: crate::TryFromSlice + crate::WriteToBuffer + grib_template_helpers::DumpField,
        {
            pub unit: table4::TimeRangeUnit,
            /// Length of the time range in units defined by `unit`.
            pub len: L,
        }
    }
}

mod table0;
mod table1;
mod table3;
mod table4;
mod table5;
mod template1;
mod template3;
mod template4;
mod template5;
