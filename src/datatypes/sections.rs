use chrono::{offset::TimeZone, DateTime, Utc};
use std::convert::TryInto;

use crate::datatypes::*;
use crate::error::*;
use crate::utils::GribInt;

macro_rules! read_as {
    ($ty:ty, $buf:ident, $start:expr) => {{
        let end = $start + std::mem::size_of::<$ty>();
        <$ty>::from_be_bytes($buf[$start..end].try_into().unwrap())
    }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indicator {
    /// Discipline - GRIB Master Table Number (see Code Table 0.0)
    pub discipline: u8,
    /// Total length of GRIB message in octets (including Section 0)
    pub total_length: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identification {
    payload: Box<[u8]>,
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

    pub fn into_slice(self) -> Box<[u8]> {
        self.payload
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

    /// Reference time of data
    #[inline]
    pub fn ref_time(&self) -> DateTime<Utc> {
        let payload = &self.payload;
        Utc.ymd(
            read_as!(u16, payload, 7).into(),
            self.payload[9].into(),
            self.payload[10].into(),
        )
        .and_hms(
            self.payload[11].into(),
            self.payload[12].into(),
            self.payload[13].into(),
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

    pub fn into_slice(self) -> Box<[u8]> {
        self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GridDefinition {
    payload: Box<[u8]>,
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

    pub fn into_slice(self) -> Box<[u8]> {
        self.payload
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProdDefinition {
    /// Number of coordinate values after Template
    pub num_coordinates: u16,
    /// Product Definition Template Number
    pub prod_tmpl_num: u16,
    pub(crate) templated: Box<[u8]>,
    pub(crate) template_supported: bool,
}

impl ProdDefinition {
    /// Use [CodeTable4_1](crate::codetables::CodeTable4_1) to get textual
    /// representation of the returned numerical value.
    pub fn parameter_category(&self) -> Option<u8> {
        if self.template_supported {
            self.templated.get(0).copied()
        } else {
            None
        }
    }

    /// Use [CodeTable4_2](crate::codetables::CodeTable4_2) to get textual
    /// representation of the returned numerical value.
    pub fn parameter_number(&self) -> Option<u8> {
        if self.template_supported {
            self.templated.get(1).copied()
        } else {
            None
        }
    }

    /// Use [CodeTable4_3](crate::codetables::CodeTable4_3) to get textual
    /// representation of the returned numerical value.
    pub fn generating_process(&self) -> Option<u8> {
        if self.template_supported {
            let index = match self.prod_tmpl_num {
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
            self.templated.get(index).copied()
        } else {
            None
        }
    }

    /// Returns the unit and value of the forecast time wrapped by `Option`.
    /// Use [CodeTable4_4](crate::codetables::CodeTable4_4) to get textual
    /// representation of the unit.
    pub fn forecast_time(&self) -> Option<ForecastTime> {
        if self.template_supported {
            let unit_index = match self.prod_tmpl_num {
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
            let unit = self.templated.get(unit_index).copied();
            let start = unit_index + 1;
            let end = unit_index + 5;
            let time = u32::from_be_bytes(self.templated[start..end].try_into().unwrap());
            unit.map(|v| ForecastTime::from_numbers(v, time))
        } else {
            None
        }
    }

    /// Returns a tuple of two [FixedSurface], wrapped by `Option`.
    pub fn fixed_surfaces(&self) -> Option<(FixedSurface, FixedSurface)> {
        if self.template_supported {
            let index = match self.prod_tmpl_num {
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
        let surface_type = self.templated.get(index).copied();
        let scale_factor = self.templated.get(index + 1).map(|v| (*v).as_grib_int());
        let start = index + 2;
        let end = index + 6;
        let scaled_value =
            u32::from_be_bytes(self.templated[start..end].try_into().unwrap()).as_grib_int();
        surface_type
            .zip(scale_factor)
            .map(|(stype, factor)| FixedSurface::new(stype, factor, scaled_value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReprDefinition {
    /// Number of data points where one or more values are
    /// specified in Section 7 when a bit map is present, total
    /// number of data points when a bit map is absent
    pub num_points: u32,
    /// Data Representation Template Number
    pub repr_tmpl_num: u16,
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
    fn prod_definition_parameters() {
        let data = ProdDefinition {
            num_coordinates: 0,
            prod_tmpl_num: 0,
            templated: vec![
                193, 0, 2, 153, 255, 0, 0, 0, 0, 0, 0, 0, 40, 1, 255, 255, 255, 255, 255, 255, 255,
                255, 255, 255, 255,
            ]
            .into_boxed_slice(),
            template_supported: true,
        };

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
