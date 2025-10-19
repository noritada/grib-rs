use std::slice::Iter;

use crate::{
    GridPointIndexIterator, PolarStereographicGridDefinition,
    codetables::SUPPORTED_PROD_DEF_TEMPLATE_NUMBERS,
    datatypes::*,
    error::*,
    grid::{
        GaussianGridDefinition, GridPointIterator, LambertGridDefinition, LatLonGridDefinition,
    },
    helpers::{GribInt, read_as},
    time::UtcDateTime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indicator {
    /// Discipline - GRIB Master Table Number (see Code Table 0.0)
    pub discipline: u8,
    /// Total length of GRIB message in octets (including Section 0)
    pub total_length: u64,
}

impl Indicator {
    pub(crate) fn from_slice(slice: &[u8]) -> Result<Self, ParseError> {
        let discipline = slice[6];
        let version = slice[7];
        if version != 2 {
            return Err(ParseError::GRIBVersionMismatch(version));
        }

        let total_length = read_as!(u64, slice, 8);

        Ok(Self {
            discipline,
            total_length,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identification {
    pub(crate) payload: Box<[u8]>,
}

impl Identification {
    pub fn from_payload(slice: Box<[u8]>) -> Result<Self, BuildError> {
        let size = slice.len();
        if size < 16 {
            Err(BuildError::SectionSizeTooSmall(size))
        } else {
            Ok(Self { payload: slice })
        }
    }

    pub fn iter(&self) -> Iter<'_, u8> {
        self.payload.iter()
    }

    /// Identification of originating/generating centre (see Common Code Table
    /// C-1)
    #[inline]
    pub fn centre_id(&self) -> u16 {
        let payload = &self.payload;
        read_as!(u16, payload, 0)
    }

    /// Identification of originating/generating sub-centre (allocated by
    /// originating/ generating centre)
    #[inline]
    pub fn subcentre_id(&self) -> u16 {
        let payload = &self.payload;
        read_as!(u16, payload, 2)
    }

    /// GRIB Master Tables Version Number (see Code Table 1.0)
    #[inline]
    pub fn master_table_version(&self) -> u8 {
        self.payload[4]
    }

    /// GRIB Local Tables Version Number (see Code Table 1.1)
    #[inline]
    pub fn local_table_version(&self) -> u8 {
        self.payload[5]
    }

    /// Significance of Reference Time (see Code Table 1.2)
    #[inline]
    pub fn ref_time_significance(&self) -> u8 {
        self.payload[6]
    }

    /// Unchecked reference time of the data.
    ///
    /// This method returns unchecked data, so for example, if the data contains
    /// a "date and time" such as "2000-13-32 25:61:62", it will be returned as
    /// is.
    pub fn ref_time_unchecked(&self) -> UtcDateTime {
        let payload = &self.payload;
        UtcDateTime::new(
            read_as!(u16, payload, 7),
            payload[9],
            payload[10],
            payload[11],
            payload[12],
            payload[13],
        )
    }

    /// Production status of processed data in this GRIB message
    /// (see Code Table 1.3)
    #[inline]
    pub fn prod_status(&self) -> u8 {
        self.payload[14]
    }

    /// Type of processed data in this GRIB message (see Code Table 1.4)
    #[inline]
    pub fn data_type(&self) -> u8 {
        self.payload[15]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUse {
    payload: Box<[u8]>,
}

impl LocalUse {
    pub fn from_payload(slice: Box<[u8]>) -> Self {
        Self { payload: slice }
    }

    pub fn iter(&self) -> Iter<'_, u8> {
        self.payload.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GridDefinition {
    pub(crate) payload: Box<[u8]>,
}

impl GridDefinition {
    pub fn from_payload(slice: Box<[u8]>) -> Result<Self, BuildError> {
        let size = slice.len();
        if size < 9 {
            Err(BuildError::SectionSizeTooSmall(size))
        } else {
            Ok(Self { payload: slice })
        }
    }

    pub fn iter(&self) -> Iter<'_, u8> {
        self.payload.iter()
    }

    /// Number of data points
    pub fn num_points(&self) -> u32 {
        let payload = &self.payload;
        read_as!(u32, payload, 1)
    }

    /// Grid Definition Template Number
    pub fn grid_tmpl_num(&self) -> u16 {
        let payload = &self.payload;
        read_as!(u16, payload, 7)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GridDefinitionTemplateValues {
    Template0(LatLonGridDefinition),
    Template20(PolarStereographicGridDefinition),
    Template30(LambertGridDefinition),
    Template40(GaussianGridDefinition),
}

impl GridDefinitionTemplateValues {
    /// Returns the shape of the grid, i.e. a tuple of the number of grids in
    /// the i and j directions.
    pub fn grid_shape(&self) -> (usize, usize) {
        match self {
            Self::Template0(def) => def.grid_shape(),
            Self::Template20(def) => def.grid_shape(),
            Self::Template30(def) => def.grid_shape(),
            Self::Template40(def) => def.grid_shape(),
        }
    }

    /// Returns the grid type.
    ///
    /// The grid types are denoted as short strings based on `gridType` used in
    /// ecCodes.
    ///
    /// This is provided primarily for debugging and simple notation purposes.
    /// It is better to use enum variants instead of the string notation to
    /// determine the grid type.
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Template0(def) => def.short_name(),
            Self::Template20(def) => def.short_name(),
            Self::Template30(def) => def.short_name(),
            Self::Template40(def) => def.short_name(),
        }
    }

    /// Returns an iterator over `(i, j)` of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        match self {
            Self::Template0(def) => def.ij(),
            Self::Template20(def) => def.ij(),
            Self::Template30(def) => def.ij(),
            Self::Template40(def) => def.ij(),
        }
    }

    /// Returns an iterator over latitudes and longitudes of grid points in
    /// degrees.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn latlons(&self) -> Result<GridPointIterator, GribError> {
        let iter = match self {
            Self::Template0(def) => GridPointIterator::LatLon(def.latlons()?),
            #[cfg(feature = "gridpoints-proj")]
            Self::Template20(def) => GridPointIterator::Lambert(def.latlons()?),
            #[cfg(feature = "gridpoints-proj")]
            Self::Template30(def) => GridPointIterator::Lambert(def.latlons()?),
            Self::Template40(def) => GridPointIterator::LatLon(def.latlons()?),
            #[cfg(not(feature = "gridpoints-proj"))]
            _ => {
                return Err(GribError::NotSupported(
                    "lat/lon computation support for the template is dropped in this build"
                        .to_owned(),
                ));
            }
        };
        Ok(iter)
    }
}

impl TryFrom<&GridDefinition> for GridDefinitionTemplateValues {
    type Error = GribError;

    fn try_from(value: &GridDefinition) -> Result<Self, Self::Error> {
        let num = value.grid_tmpl_num();
        match num {
            0 => {
                let buf = &value.payload;
                if buf.len() > 67 {
                    return Err(GribError::NotSupported(format!(
                        "template {num} with list of number of points"
                    )));
                }
                Ok(GridDefinitionTemplateValues::Template0(
                    LatLonGridDefinition::from_buf(&buf[25..]),
                ))
            }
            20 => {
                let buf = &value.payload;
                Ok(GridDefinitionTemplateValues::Template20(
                    PolarStereographicGridDefinition::from_buf(&buf[9..]),
                ))
            }
            30 => {
                let buf = &value.payload;
                Ok(GridDefinitionTemplateValues::Template30(
                    LambertGridDefinition::from_buf(&buf[9..]),
                ))
            }
            40 => {
                let buf = &value.payload;
                if buf.len() > 67 {
                    return Err(GribError::NotSupported(format!(
                        "template {num} with list of number of points"
                    )));
                }
                Ok(GridDefinitionTemplateValues::Template40(
                    GaussianGridDefinition::from_buf(&buf[25..]),
                ))
            }
            _ => Err(GribError::NotSupported(format!("template {num}"))),
        }
    }
}

const START_OF_PROD_TEMPLATE: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProdDefinition {
    pub(crate) payload: Box<[u8]>,
}

impl ProdDefinition {
    pub fn from_payload(slice: Box<[u8]>) -> Result<Self, BuildError> {
        let size = slice.len();
        if size < START_OF_PROD_TEMPLATE {
            Err(BuildError::SectionSizeTooSmall(size))
        } else {
            Ok(Self { payload: slice })
        }
    }

    pub fn iter(&self) -> Iter<'_, u8> {
        self.payload.iter()
    }

    /// Number of coordinate values after Template
    pub fn num_coordinates(&self) -> u16 {
        let payload = &self.payload;
        read_as!(u16, payload, 0)
    }

    /// Product Definition Template Number
    pub fn prod_tmpl_num(&self) -> u16 {
        let payload = &self.payload;
        read_as!(u16, payload, 2)
    }

    // pub(crate) templated(&self)-> Box<[u8]> {

    // }

    pub(crate) fn template_supported(&self) -> bool {
        SUPPORTED_PROD_DEF_TEMPLATE_NUMBERS.contains(&self.prod_tmpl_num())
    }

    /// Use [CodeTable4_1](crate::codetables::CodeTable4_1) to get textual
    /// representation of the returned numerical value.
    pub fn parameter_category(&self) -> Option<u8> {
        if self.template_supported() {
            self.payload.get(START_OF_PROD_TEMPLATE).copied()
        } else {
            None
        }
    }

    /// Use [CodeTable4_2](crate::codetables::CodeTable4_2) to get textual
    /// representation of the returned numerical value.
    pub fn parameter_number(&self) -> Option<u8> {
        if self.template_supported() {
            self.payload.get(START_OF_PROD_TEMPLATE + 1).copied()
        } else {
            None
        }
    }

    /// Use [CodeTable4_3](crate::codetables::CodeTable4_3) to get textual
    /// representation of the returned numerical value.
    pub fn generating_process(&self) -> Option<u8> {
        if self.template_supported() {
            let index = match self.prod_tmpl_num() {
                0..=39 => Some(2),
                40..=43 => Some(4),
                44..=46 => Some(15),
                47 => Some(2),
                48..=49 => Some(26),
                51 => Some(2),
                // 53 and 54 is variable and not supported as of now
                55..=56 => Some(8),
                // 57 and 58 is variable and not supported as of now
                59 => Some(8),
                60..=61 => Some(2),
                62..=63 => Some(8),
                // 67 and 68 is variable and not supported as of now
                70..=73 => Some(7),
                76..=79 => Some(5),
                80..=81 => Some(27),
                82 => Some(16),
                83 => Some(2),
                84 => Some(16),
                85 => Some(15),
                86..=91 => Some(2),
                254 => Some(2),
                1000..=1101 => Some(2),
                _ => None,
            }?;
            self.payload.get(START_OF_PROD_TEMPLATE + index).copied()
        } else {
            None
        }
    }

    /// Returns the unit and value of the forecast time wrapped by `Option`.
    /// Use [CodeTable4_4](crate::codetables::CodeTable4_4) to get textual
    /// representation of the unit.
    pub fn forecast_time(&self) -> Option<ForecastTime> {
        if self.template_supported() {
            let unit_index = match self.prod_tmpl_num() {
                0..=15 => Some(8),
                32..=34 => Some(8),
                40..=43 => Some(10),
                44..=47 => Some(21),
                48..=49 => Some(32),
                51 => Some(8),
                // 53 and 54 is variable and not supported as of now
                55..=56 => Some(14),
                // 57 and 58 is variable and not supported as of now
                59 => Some(14),
                60..=61 => Some(8),
                62..=63 => Some(14),
                // 67 and 68 is variable and not supported as of now
                70..=73 => Some(13),
                76..=79 => Some(11),
                80..=81 => Some(33),
                82..=84 => Some(22),
                85 => Some(21),
                86..=87 => Some(8),
                88 => Some(26),
                91 => Some(8),
                1000..=1101 => Some(8),
                _ => None,
            }?;
            let unit_index = START_OF_PROD_TEMPLATE + unit_index;
            let unit = self.payload.get(unit_index).copied();
            let start = unit_index + 1;
            let end = unit_index + 5;
            let time = u32::from_be_bytes(self.payload[start..end].try_into().unwrap());
            unit.map(|v| ForecastTime::from_numbers(v, time))
        } else {
            None
        }
    }

    /// Returns a tuple of two [FixedSurface], wrapped by `Option`.
    pub fn fixed_surfaces(&self) -> Option<(FixedSurface, FixedSurface)> {
        if self.template_supported() {
            let index = match self.prod_tmpl_num() {
                0..=15 => Some(13),
                40..=43 => Some(15),
                44 => Some(24),
                45..=47 => Some(26),
                48..=49 => Some(37),
                51 => Some(13),
                // 53 and 54 is variable and not supported as of now
                55..=56 => Some(19),
                // 57 and 58 is variable and not supported as of now
                59 => Some(19),
                60..=61 => Some(13),
                62..=63 => Some(19),
                // 67 and 68 is variable and not supported as of now
                70..=73 => Some(18),
                76..=79 => Some(16),
                80..=81 => Some(38),
                82..=84 => Some(27),
                85 => Some(26),
                86..=87 => Some(13),
                88 => Some(5),
                91 => Some(13),
                1100..=1101 => Some(13),
                _ => None,
            }?;

            let first_surface = self.read_surface_from(index);
            let second_surface = self.read_surface_from(index + 6);
            first_surface.zip(second_surface)
        } else {
            None
        }
    }

    fn read_surface_from(&self, index: usize) -> Option<FixedSurface> {
        let index = START_OF_PROD_TEMPLATE + index;
        let surface_type = self.payload.get(index).copied();
        let scale_factor = self.payload.get(index + 1).map(|v| (*v).as_grib_int());
        let start = index + 2;
        let end = index + 6;
        let scaled_value =
            u32::from_be_bytes(self.payload[start..end].try_into().unwrap()).as_grib_int();
        surface_type
            .zip(scale_factor)
            .map(|(stype, factor)| FixedSurface::new(stype, factor, scaled_value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReprDefinition {
    pub(crate) payload: Box<[u8]>,
}

impl ReprDefinition {
    pub fn from_payload(slice: Box<[u8]>) -> Result<Self, BuildError> {
        let size = slice.len();
        if size < 6 {
            Err(BuildError::SectionSizeTooSmall(size))
        } else {
            Ok(Self { payload: slice })
        }
    }

    pub fn iter(&self) -> Iter<'_, u8> {
        self.payload.iter()
    }

    /// Number of data points where one or more values are
    /// specified in Section 7 when a bit map is present, total
    /// number of data points when a bit map is absent
    pub fn num_points(&self) -> u32 {
        let payload = &self.payload;
        read_as!(u32, payload, 0)
    }

    /// Data Representation Template Number
    pub fn repr_tmpl_num(&self) -> u16 {
        let payload = &self.payload;
        read_as!(u16, payload, 4)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitMap {
    /// Bit-map indicator
    pub bitmap_indicator: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_definition_template_0() {
        // data taken from submessage #0.0 of
        // `Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz`
        // in `testdata`
        let data = GridDefinition::from_payload(
            vec![
                0x00, 0x00, 0x01, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xff, 0xff, 0xff, 0xff,
                0xff, 0x01, 0x03, 0xcd, 0x39, 0xfa, 0x01, 0x03, 0xc9, 0xf6, 0xa3, 0x00, 0x00, 0x01,
                0x00, 0x00, 0x00, 0x01, 0x50, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0x02,
                0xdb, 0xc9, 0x3d, 0x07, 0x09, 0x7d, 0xa4, 0x30, 0x01, 0x31, 0xcf, 0xc3, 0x08, 0xef,
                0xdd, 0x5c, 0x00, 0x01, 0xe8, 0x48, 0x00, 0x01, 0x45, 0x85, 0x00,
            ]
            .into_boxed_slice(),
        )
        .unwrap();

        let actual = GridDefinitionTemplateValues::try_from(&data).unwrap();
        let expected = GridDefinitionTemplateValues::Template0(LatLonGridDefinition {
            ni: 256,
            nj: 336,
            first_point_lat: 47958333,
            first_point_lon: 118062500,
            last_point_lat: 20041667,
            last_point_lon: 149937500,
            scanning_mode: crate::grid::ScanningMode(0b00000000),
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn prod_definition_parameters() {
        let data = ProdDefinition::from_payload(
            vec![
                0, 0, 0, 0, 193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 40, 1, 255, 255, 255, 255,
                255, 255, 255, 255, 255, 255, 255,
            ]
            .into_boxed_slice(),
        )
        .unwrap();

        assert_eq!(data.parameter_category(), Some(193));
        assert_eq!(data.parameter_number(), Some(0));
        assert_eq!(
            data.forecast_time(),
            Some(ForecastTime::from_numbers(0, 40))
        );
        assert_eq!(
            data.fixed_surfaces(),
            Some((
                FixedSurface::new(1, -127, -2147483647),
                FixedSurface::new(255, -127, -2147483647)
            ))
        );
    }
}
