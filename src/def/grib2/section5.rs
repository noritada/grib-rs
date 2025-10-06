use grib_template_derive::{Dump, TryFromSlice};

use super::template5;

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
    SimplePacking(template5::SimplePacking) = 0,
    SimplePackingForMatrix(template5::SimplePackingForMatrix) = 1,
    ComplexPacking(template5::ComplexPacking) = 2,
    ComplexPackingWithSpatialDiff(template5::ComplexPackingWithSpatialDiff) = 3,
    IeeeFloatingPoints(template5::IeeeFloatingPoints) = 4,
    Jpeg2000CodeStream(template5::Jpeg2000CodeStream) = 40,
    Png(template5::Png) = 41,
    CcsdsLosslessPacking(template5::CcsdsLosslessPacking) = 42,
    SimplePackingForSpectral(template5::SimplePackingForSpectral) = 50,
    ComplexPackingForSphericalHarmonics(template5::ComplexPackingForSphericalHarmonics) = 51,
    ComplexPackingForAreaModelsSpectral(template5::ComplexPackingForAreaModelsSpectral) = 53,
    SimplePackingWithLogarithmPreprocessing(template5::SimplePackingWithLogarithmPreprocessing) =
        61,
    RunLengthPacking(template5::RunLengthPacking) = 200,
}
