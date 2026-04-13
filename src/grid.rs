use grib_template_helpers::TryFromSlice;
use helpers::RegularGridIterator;

pub use self::{gaussian::compute_gaussian_latitudes, rotated_ll::Unrotate};
use crate::{
    GribError, GridDefinition,
    def::grib2::template::{
        Template3_0, Template3_1, Template3_20, Template3_30, Template3_40,
        param_set::{Grid, ScanningMode},
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

impl GridShortName for GridDefinitionTemplateValues {
    fn short_name(&self) -> &'static str {
        match self {
            Self::Template0(def) => def.lat_lon.short_name(),
            Self::Template1(def) => def.short_name(),
            Self::Template20(def) => def.short_name(),
            Self::Template30(def) => def.short_name(),
            Self::Template40(def) => def.gaussian.short_name(),
        }
    }
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

impl LatLons for GridDefinitionTemplateValues {
    type Iter<'a>
        = GridPointLatLons
    where
        Self: 'a;

    fn latlons_unchecked<'a>(&'a self) -> Result<Self::Iter<'a>, GribError> {
        let iter = match self {
            Self::Template0(def) => GridPointLatLons::from(def.lat_lon.latlons_unchecked()?),
            Self::Template1(def) => GridPointLatLons::from(def.latlons_unchecked()?),
            #[cfg(feature = "gridpoints-proj")]
            Self::Template20(def) => GridPointLatLons::from(def.latlons_unchecked()?),
            #[cfg(feature = "gridpoints-proj")]
            Self::Template30(def) => GridPointLatLons::from(def.latlons_unchecked()?),
            Self::Template40(def) => GridPointLatLons::from(def.gaussian.latlons_unchecked()?),
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

/// A functionality to return a short name of the grid system.
pub trait GridShortName {
    /// Returns the grid type.
    ///
    /// The grid types are denoted as short strings based on `gridType` used in
    /// ecCodes.
    ///
    /// This is provided primarily for debugging and simple notation purposes.
    /// It is better to use enum variants instead of the string notation to
    /// determine the grid type.
    fn short_name(&self) -> &'static str;
}

/// A functionality to generate an iterator over latitude/longitude of grid
/// points.
pub trait LatLons {
    type Iter<'a>: Iterator<Item = (f32, f32)>
    where
        Self: 'a;

    /// Computes and returns an iterator over latitudes and longitudes of grid
    /// points in degrees. Unlike the return values from [`LatLons::latlons`],
    /// the longitude values from thie method do not necessarily fall within
    /// the range `[-180°, 180°]`.
    fn latlons_unchecked<'a>(&'a self) -> Result<Self::Iter<'a>, GribError>;

    /// Computes and returns an iterator over latitudes and longitudes of grid
    /// points in degrees. The returned longitude values are converted to fall
    /// within the range `[-180°, 180°]`.
    ///
    /// The order of lat/lon data of grid points is the same as the order of the
    /// grid point values, defined by the scanning mode
    /// ([`ScanningMode`](`crate::def::grib2::template::param_set::ScanningMode`)) in the data.
    #[allow(clippy::type_complexity)]
    fn latlons<'a>(
        &'a self,
    ) -> Result<std::iter::Map<Self::Iter<'a>, fn((f32, f32)) -> (f32, f32)>, GribError> {
        let iter = self
            .latlons_unchecked()?
            .map(helpers::normalize_latlon as fn((f32, f32)) -> (f32, f32));
        Ok(iter)
    }
}

/// An iterator over latitudes and longitudes of grid points in a submessage.
///
/// This `struct` is created by the [`latlons`] method on [`LatLons`]
/// implemented for [`SubMessage`]. See its documentation for more.
///
/// [`latlons`]: crate::context::SubMessage::latlons
/// [`SubMessage`]: crate::context::SubMessage
#[derive(Clone)]
pub struct GridPointLatLons(LatLonsWrapper);

impl Iterator for GridPointLatLons {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self(LatLonsWrapper::SigR(iter)) => iter.next(),
            Self(LatLonsWrapper::SigUR(iter)) => iter.next(),
            Self(LatLonsWrapper::SigIf(iter)) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self(LatLonsWrapper::SigR(iter)) => iter.size_hint(),
            Self(LatLonsWrapper::SigUR(iter)) => iter.size_hint(),
            Self(LatLonsWrapper::SigIf(iter)) => iter.size_hint(),
        }
    }
}

impl From<RegularGridIterator> for GridPointLatLons {
    fn from(value: RegularGridIterator) -> Self {
        Self(LatLonsWrapper::SigR(value))
    }
}

impl From<Unrotate<RegularGridIterator>> for GridPointLatLons {
    fn from(value: Unrotate<RegularGridIterator>) -> Self {
        Self(LatLonsWrapper::SigUR(value))
    }
}

impl From<std::vec::IntoIter<(f32, f32)>> for GridPointLatLons {
    fn from(value: std::vec::IntoIter<(f32, f32)>) -> Self {
        Self(LatLonsWrapper::SigIf(value))
    }
}

#[derive(Clone)]
enum LatLonsWrapper {
    SigR(RegularGridIterator),
    SigUR(Unrotate<RegularGridIterator>),
    SigIf(std::vec::IntoIter<(f32, f32)>),
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

pub(crate) trait AngleUnit {
    fn angle_unit(&self) -> f64;
}

impl AngleUnit for Grid {
    fn angle_unit(&self) -> f64 {
        let basic_angle = self.initial_production_domain_basic_angle;
        let sub_angle = self.basic_angle_subdivisions;
        if basic_angle == 0 {
            1e-6
        } else {
            basic_angle as f64 / sub_angle as f64
        }
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
