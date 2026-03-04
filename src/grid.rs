use grib_template_helpers::TryFromSlice;
use helpers::RegularGridIterator;

pub use self::{gaussian::compute_gaussian_latitudes, rotated_ll::Unrotate};
use crate::{
    GribError, GridDefinition,
    def::grib2::template::{
        Template3_0, Template3_1, Template3_20, Template3_30, Template3_40, param_set::ScanningMode,
    },
};

#[derive(Debug, PartialEq)]
pub enum GridDefinitionTemplateValues {
    Template0(Template3_0),
    Template1(Template3_1),
    Template20(Template3_20),
    Template30(Template3_30),
    Template40(Template3_40),
}

impl GridPointIndex for GridDefinitionTemplateValues {
    fn grid_shape(&self) -> (usize, usize) {
        match self {
            Self::Template0(def) => def.lat_lon.grid_shape(),
            Self::Template1(def) => def.grid_shape(),
            Self::Template20(def) => def.grid_shape(),
            Self::Template30(def) => def.grid_shape(),
            Self::Template40(def) => def.gaussian.grid_shape(),
        }
    }

    fn scanning_mode(&self) -> &ScanningMode {
        match self {
            Self::Template0(def) => def.lat_lon.scanning_mode(),
            Self::Template1(def) => def.scanning_mode(),
            Self::Template20(def) => def.scanning_mode(),
            Self::Template30(def) => def.scanning_mode(),
            Self::Template40(def) => def.gaussian.scanning_mode(),
        }
    }
}

impl GridDefinitionTemplateValues {
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
            Self::Template0(def) => def.lat_lon.short_name(),
            Self::Template1(def) => def.short_name(),
            Self::Template20(def) => def.short_name(),
            Self::Template30(def) => def.short_name(),
            Self::Template40(def) => def.gaussian.short_name(),
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
            Self::Template0(def) => GridPointIterator::LatLon(def.lat_lon.latlons()?),
            Self::Template1(def) => GridPointIterator::RotatedLatLon(def.latlons()?),
            #[cfg(feature = "gridpoints-proj")]
            Self::Template20(def) => GridPointIterator::Lambert(def.latlons()?),
            #[cfg(feature = "gridpoints-proj")]
            Self::Template30(def) => GridPointIterator::Lambert(def.latlons()?),
            Self::Template40(def) => GridPointIterator::LatLon(def.gaussian.latlons()?),
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
        // In the future, we should switch the implementation like this:
        //
        // ```
        // let buf = &value.payload;
        // let mut pos = 0;
        // let payload = crate::def::grib2::Section3Payload::try_from_slice(buf, &mut pos)
        //     .map_err(|e| GribError::Unknown(e.to_owned()))?;
        // let template = match payload.template {
        // ..
        // }
        // ```
        //
        // However, since the current implementation of the templates has many
        // limitations, to prevent errors, the template reading process is implemented
        // as follows.
        let buf = &value.payload[9..];
        let mut pos = 0;
        let num = value.grid_tmpl_num();
        let template = match num {
            0 => GridDefinitionTemplateValues::Template0(
                Template3_0::try_from_slice(buf, &mut pos)
                    .map_err(|e| GribError::Unknown(e.to_owned()))?,
            ),
            1 => GridDefinitionTemplateValues::Template1(
                Template3_1::try_from_slice(buf, &mut pos)
                    .map_err(|e| GribError::Unknown(e.to_owned()))?,
            ),
            20 => GridDefinitionTemplateValues::Template20(
                Template3_20::try_from_slice(buf, &mut pos)
                    .map_err(|e| GribError::Unknown(e.to_owned()))?,
            ),
            30 => GridDefinitionTemplateValues::Template30(
                Template3_30::try_from_slice(buf, &mut pos)
                    .map_err(|e| GribError::Unknown(e.to_owned()))?,
            ),
            40 => GridDefinitionTemplateValues::Template40(
                Template3_40::try_from_slice(buf, &mut pos)
                    .map_err(|e| GribError::Unknown(e.to_owned()))?,
            ),
            _ => {
                return Err(GribError::NotSupported(format!(
                    "lat/lon computation support for the template {num} is dropped in this build"
                )));
            }
        };
        if buf.len() > pos {
            return Err(GribError::NotSupported(
                "template with list of number of points".to_owned(),
            ));
        }
        Ok(template)
    }
}

/// An iterator over latitudes and longitudes of grid points in a submessage.
///
/// This `enum` is created by the [`latlons`] method on [`SubMessage`]. See its
/// documentation for more.
///
/// [`latlons`]: crate::context::SubMessage::latlons
/// [`SubMessage`]: crate::context::SubMessage
#[derive(Clone)]
pub enum GridPointIterator {
    LatLon(RegularGridIterator),
    RotatedLatLon(Unrotate<RegularGridIterator>),
    Lambert(std::vec::IntoIter<(f32, f32)>),
}

impl Iterator for GridPointIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::LatLon(iter) => iter.next(),
            Self::RotatedLatLon(iter) => iter.next(),
            Self::Lambert(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::LatLon(iter) => iter.size_hint(),
            Self::RotatedLatLon(iter) => iter.size_hint(),
            Self::Lambert(iter) => iter.size_hint(),
        }
    }
}

/// A functionality to generate an iterator over 2D index `(i, j)` of grid
/// points.
///
/// # Examples
///
/// ```
/// use grib::{GridPointIndex, def::grib2::template::param_set::ScanningMode};
///
/// struct Grid {
///     ni: u32,
///     nj: u32,
///     scanning_mode: ScanningMode,
/// }
///
/// impl GridPointIndex for Grid {
///     fn grid_shape(&self) -> (usize, usize) {
///         (self.ni as usize, self.nj as usize)
///     }
///
///     fn scanning_mode(&self) -> &ScanningMode {
///         &self.scanning_mode
///     }
/// }
///
/// let grid_2x3 = Grid {
///     ni: 2,
///     nj: 3,
///     scanning_mode: ScanningMode(0b01000000),
/// };
/// assert_eq!(grid_2x3.grid_shape(), (2, 3));
///
/// let mut iter = grid_2x3.ij().unwrap();
/// assert_eq!(iter.next(), Some((0, 0)));
/// assert_eq!(iter.next(), Some((1, 0)));
/// assert_eq!(iter.next(), Some((0, 1)));
/// assert_eq!(iter.next(), Some((1, 1)));
/// assert_eq!(iter.next(), Some((0, 2)));
/// assert_eq!(iter.next(), Some((1, 2)));
/// assert_eq!(iter.next(), None);
/// ```
pub trait GridPointIndex {
    /// Returns the shape of the grid, i.e. a tuple of the number of grids in
    /// the i and j directions.
    fn grid_shape(&self) -> (usize, usize);

    /// Returns [`ScanningMode`] used for the iteration.
    fn scanning_mode(&self) -> &ScanningMode;

    /// Returns an iterator over 2D index `(i, j)` of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        GridPointIndexIterator::new(self.grid_shape(), *self.scanning_mode())
    }
}

/// An iterator over 2D index `(i, j)` of grid points.
///
/// This `struct` is created by the [`GridPointIndex::ij`] method. See its
/// documentation for more.
#[derive(Clone)]
pub struct GridPointIndexIterator {
    major_len: usize,
    minor_len: usize,
    scanning_mode: ScanningMode,
    major_pos: usize,
    minor_pos: usize,
    increments: bool,
}

impl GridPointIndexIterator {
    pub(crate) fn new(
        (i_len, j_len): (usize, usize),
        scanning_mode: ScanningMode,
    ) -> Result<Self, GribError> {
        if scanning_mode.has_unsupported_flags() {
            let ScanningMode(mode) = scanning_mode;
            return Err(GribError::NotSupported(format!("scanning mode {mode}")));
        }

        let (major_len, minor_len) = if scanning_mode.is_consecutive_for_i() {
            (j_len, i_len)
        } else {
            (i_len, j_len)
        };

        Ok(Self {
            major_len,
            minor_len,
            scanning_mode,
            minor_pos: 0,
            major_pos: 0,
            increments: true,
        })
    }
}

impl Iterator for GridPointIndexIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.major_pos == self.major_len {
            return None;
        }

        let minor = if self.increments {
            self.minor_pos
        } else {
            self.minor_len - self.minor_pos - 1
        };
        let major = self.major_pos;

        self.minor_pos += 1;
        if self.minor_pos == self.minor_len {
            self.major_pos += 1;
            self.minor_pos = 0;
            if self.scanning_mode.scans_alternating_rows() {
                self.increments = !self.increments;
            }
        }

        if self.scanning_mode.is_consecutive_for_i() {
            Some((minor, major))
        } else {
            Some((major, minor))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.major_len - self.major_pos) * self.minor_len - self.minor_pos;
        (len, Some(len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::def::grib2::template::param_set::EarthShape;

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
        let expected = GridDefinitionTemplateValues::Template0(Template3_0 {
            earth: EarthShape {
                shape: 4,
                spherical_earth_radius_scale_factor: 0xff,
                spherical_earth_radius_scaled_value: 0xffffffff,
                major_axis_scale_factor: 1,
                major_axis_scaled_value: 63781370,
                minor_axis_scale_factor: 1,
                minor_axis_scaled_value: 63567523,
            },
            lat_lon: crate::def::grib2::template::param_set::LatLonGrid {
                grid: crate::def::grib2::template::param_set::Grid {
                    ni: 256,
                    nj: 336,
                    initial_production_domain_basic_angle: 0,
                    basic_angle_subdivisions: 0xffffffff,
                    first_point_lat: 47958333,
                    first_point_lon: 118062500,
                    resolution_and_component_flags:
                        crate::def::grib2::template::param_set::ResolutionAndComponentFlags(
                            0b00110000,
                        ),
                    last_point_lat: 20041667,
                    last_point_lon: 149937500,
                },
                i_direction_inc: 125000,
                j_direction_inc: 83333,
                scanning_mode: crate::def::grib2::template::param_set::ScanningMode(0b00000000),
            },
        });
        assert_eq!(actual, expected);
    }
}

mod earth;
mod flags;
mod gaussian;
mod helpers;
mod lambert;
mod latlon;
mod polar_stereographic;
mod rotated_ll;
