use proj::Proj;

pub use self::earth::EarthShapeDefinition;
use crate::{
    error::GribError,
    utils::{read_as, GribInt},
};

/// An iterator over latitudes and longitudes of grid points in a submessage.
///
/// This `enum` is created by the [`latlons`] method on [`SubMessage`]. See its
/// documentation for more.
///
/// [`latlons`]: crate::context::SubMessage::latlons
/// [`SubMessage`]: crate::context::SubMessage
#[derive(Clone)]
pub enum GridPointIterator {
    LatLon(LatLonGridIterator),
    Lambert(std::vec::IntoIter<(f32, f32)>),
}

impl Iterator for GridPointIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::LatLon(iter) => iter.next(),
            Self::Lambert(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::LatLon(iter) => iter.size_hint(),
            Self::Lambert(iter) => iter.size_hint(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LatLonGridDefinition {
    pub ni: u32,
    pub nj: u32,
    pub first_point_lat: i32,
    pub first_point_lon: i32,
    pub last_point_lat: i32,
    pub last_point_lon: i32,
    pub scanning_mode: ScanningMode,
}

impl LatLonGridDefinition {
    /// Returns an iterator over `(i, j)` of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    ///
    /// Examples
    ///
    /// ```
    /// let def = grib::LatLonGridDefinition {
    ///     ni: 2,
    ///     nj: 3,
    ///     first_point_lat: 0,
    ///     first_point_lon: 0,
    ///     last_point_lat: 2_000_000,
    ///     last_point_lon: 1_000_000,
    ///     scanning_mode: grib::ScanningMode(0b01000000),
    /// };
    /// let ij = def.ij();
    /// assert!(ij.is_ok());
    ///
    /// let mut ij = ij.unwrap();
    /// assert_eq!(ij.next(), Some((0, 0)));
    /// assert_eq!(ij.next(), Some((1, 0)));
    /// assert_eq!(ij.next(), Some((0, 1)));
    /// ```
    pub fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        if self.scanning_mode.has_unsupported_flags() {
            let ScanningMode(mode) = self.scanning_mode;
            return Err(GribError::NotSupported(format!("scanning mode {mode}")));
        }

        let iter =
            GridPointIndexIterator::new(self.ni as usize, self.nj as usize, self.scanning_mode);
        Ok(iter)
    }

    /// Returns an iterator over latitudes and longitudes of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    ///
    /// Examples
    ///
    /// ```
    /// let def = grib::LatLonGridDefinition {
    ///     ni: 2,
    ///     nj: 3,
    ///     first_point_lat: 0,
    ///     first_point_lon: 0,
    ///     last_point_lat: 2_000_000,
    ///     last_point_lon: 1_000_000,
    ///     scanning_mode: grib::ScanningMode(0b01000000),
    /// };
    /// let latlons = def.latlons();
    /// assert!(latlons.is_ok());
    ///
    /// let mut latlons = latlons.unwrap();
    /// assert_eq!(latlons.next(), Some((0.0, 0.0)));
    /// assert_eq!(latlons.next(), Some((0.0, 1.0)));
    /// assert_eq!(latlons.next(), Some((1.0, 0.0)));
    /// ```
    pub fn latlons(&self) -> Result<LatLonGridIterator, GribError> {
        if !self.is_consistent() {
            return Err(GribError::InvalidValueError("Latitude and longitude for first/last grid points are not consistent with scanning mode".to_owned()));
        }

        let ij = self.ij()?;

        let lat_diff = self.last_point_lat - self.first_point_lat;
        let lon_diff = self.last_point_lon - self.first_point_lon;
        let (num_div_lat, num_div_lon) = ((self.nj - 1) as i32, (self.ni - 1) as i32);
        let lat_delta = lat_diff as f32 / num_div_lat as f32;
        let lat = (0..=num_div_lat)
            .map(|x| (self.first_point_lat as f32 + x as f32 * lat_delta) / 1_000_000_f32)
            .collect();
        let lon_delta = lon_diff as f32 / num_div_lon as f32;
        let lon = (0..=num_div_lon)
            .map(|x| (self.first_point_lon as f32 + x as f32 * lon_delta) / 1_000_000_f32)
            .collect();

        let iter = LatLonGridIterator::new(lat, lon, ij);
        Ok(iter)
    }

    fn is_consistent(&self) -> bool {
        let lat_diff = self.last_point_lat - self.first_point_lat;
        let lon_diff = self.last_point_lon - self.first_point_lon;
        !(((lat_diff > 0) ^ self.scanning_mode.scans_positively_for_j())
            || ((lon_diff > 0) ^ self.scanning_mode.scans_positively_for_i()))
    }

    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let ni = read_as!(u32, buf, 0);
        let nj = read_as!(u32, buf, 4);
        let first_point_lat = read_as!(u32, buf, 16).as_grib_int();
        let first_point_lon = read_as!(u32, buf, 20).as_grib_int();
        let last_point_lat = read_as!(u32, buf, 25).as_grib_int();
        let last_point_lon = read_as!(u32, buf, 29).as_grib_int();
        let scanning_mode = read_as!(u8, buf, 41);
        Self {
            ni,
            nj,
            first_point_lat,
            first_point_lon,
            last_point_lat,
            last_point_lon,
            scanning_mode: ScanningMode(scanning_mode),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LambertGridDefinition {
    pub earth_shape: EarthShapeDefinition,
    pub ni: u32,
    pub nj: u32,
    pub first_point_lat: i32,
    pub first_point_lon: i32,
    pub lad: i32,
    pub lov: i32,
    pub dx: u32,
    pub dy: u32,
    pub scanning_mode: ScanningMode,
    pub latin1: i32,
    pub latin2: i32,
}

impl LambertGridDefinition {
    /// Returns an iterator over `(i, j)` of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    ///
    /// Examples
    ///
    /// ```
    /// let def = grib::LambertGridDefinition {
    ///     earth_shape: grib::EarthShapeDefinition {
    ///         shape_of_the_earth: 1,
    ///         scale_factor_of_radius_of_spherical_earth: 0,
    ///         scaled_value_of_radius_of_spherical_earth: 6371200,
    ///         scale_factor_of_earth_major_axis: 0,
    ///         scaled_value_of_earth_major_axis: 0,
    ///         scale_factor_of_earth_minor_axis: 0,
    ///         scaled_value_of_earth_minor_axis: 0,
    ///     },
    ///     ni: 2,
    ///     nj: 3,
    ///     first_point_lat: 0,
    ///     first_point_lon: 0,
    ///     lad: 0,
    ///     lov: 0,
    ///     dx: 1000,
    ///     dy: 1000,
    ///     scanning_mode: grib::ScanningMode(0b01000000),
    ///     latin1: 0,
    ///     latin2: 0,
    /// };
    /// let ij = def.ij();
    /// assert!(ij.is_ok());
    ///
    /// let mut ij = ij.unwrap();
    /// assert_eq!(ij.next(), Some((0, 0)));
    /// assert_eq!(ij.next(), Some((1, 0)));
    /// assert_eq!(ij.next(), Some((0, 1)));
    /// ```
    pub fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        if self.scanning_mode.has_unsupported_flags() {
            let ScanningMode(mode) = self.scanning_mode;
            return Err(GribError::NotSupported(format!("scanning mode {mode}")));
        }

        let iter =
            GridPointIndexIterator::new(self.ni as usize, self.nj as usize, self.scanning_mode);
        Ok(iter)
    }

    /// Returns an iterator over latitudes and longitudes of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn latlons(&self) -> Result<std::vec::IntoIter<(f32, f32)>, GribError> {
        let lad = self.lad as f64 * 1e-6;
        let lov = self.lov as f64 * 1e-6;
        let latin1 = self.latin1 as f64 * 1e-6;
        let latin2 = self.latin2 as f64 * 1e-6;
        let (a, b) = self.earth_shape.radii().ok_or_else(|| {
            GribError::NotSupported(format!(
                "unknown value of Code Table 3.2 (shape of the Earth): {}",
                self.earth_shape.shape_of_the_earth
            ))
        })?;
        let proj_def = format!(
            "+a={a} +b={b} +proj=lcc +lat_0={lad} +lon_0={lov} +lat_1={latin1} +lat_2={latin2}"
        );
        let projection = Proj::new(&proj_def).map_err(|e| GribError::Unknown(e.to_string()))?;
        let (first_corner_x, first_corner_y) = projection
            .project(
                (
                    (self.first_point_lon as f64 * 1e-6).to_radians(),
                    (self.first_point_lat as f64 * 1e-6).to_radians(),
                ),
                false,
            )
            .map_err(|e| GribError::Unknown(e.to_string()))?;

        let dx = self.dx as f64 * 1e-3;
        let dy = self.dy as f64 * 1e-3;
        let dx = if !self.scanning_mode.scans_positively_for_i() && dx > 0. {
            -dx
        } else {
            dx
        };
        let dy = if !self.scanning_mode.scans_positively_for_j() && dy > 0. {
            -dy
        } else {
            dy
        };

        let ij = self.ij()?;
        let mut ij = ij
            .map(|(i, j)| {
                (
                    first_corner_x + dx * i as f64,
                    first_corner_y + dy * j as f64,
                )
            })
            .collect::<Vec<_>>();

        let lonlat = projection
            .project_array(&mut ij, true)
            .map_err(|e| GribError::Unknown(e.to_string()))?;
        let latlon = lonlat
            .iter_mut()
            .map(|(lon, lat)| (lat.to_degrees() as f32, lon.to_degrees() as f32))
            .collect::<Vec<_>>();

        Ok(latlon.into_iter())
    }

    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let earth_shape = EarthShapeDefinition::from_buf(buf);
        let ni = read_as!(u32, buf, 16);
        let nj = read_as!(u32, buf, 20);
        let first_point_lat = read_as!(u32, buf, 24).as_grib_int();
        let first_point_lon = read_as!(u32, buf, 28).as_grib_int();
        let lad = read_as!(u32, buf, 33).as_grib_int();
        let lov = read_as!(u32, buf, 37).as_grib_int();
        let dx = read_as!(u32, buf, 41);
        let dy = read_as!(u32, buf, 45);
        let scanning_mode = read_as!(u8, buf, 50);
        let latin1 = read_as!(u32, buf, 51).as_grib_int();
        let latin2 = read_as!(u32, buf, 55).as_grib_int();
        Self {
            earth_shape,
            ni,
            nj,
            first_point_lat,
            first_point_lon,
            lad,
            lov,
            dx,
            dy,
            scanning_mode: ScanningMode(scanning_mode),
            latin1,
            latin2,
        }
    }
}

/// An iterator over `(i, j)` of grid points.
///
/// This `struct` is created by the [`ij`] method. See its documentation for
/// more.
///
/// [`ij`]: LatLonGridDefinition::ij
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
    pub(crate) fn new(i_len: usize, j_len: usize, scanning_mode: ScanningMode) -> Self {
        let (major_len, minor_len) = if scanning_mode.is_consecutive_for_i() {
            (j_len, i_len)
        } else {
            (i_len, j_len)
        };

        Self {
            major_len,
            minor_len,
            scanning_mode,
            minor_pos: 0,
            major_pos: 0,
            increments: true,
        }
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

/// An iterator over latitudes and longitudes of grid points of a lat/lon grid.
///
/// This `struct` is created by the [`latlons`] method on
/// [`LatLonGridDefinition`]. See its documentation for more.
///
/// [`latlons`]: LatLonGridDefinition::latlons
#[derive(Clone)]
pub struct LatLonGridIterator {
    lat: Vec<f32>,
    lon: Vec<f32>,
    ij: GridPointIndexIterator,
}

impl LatLonGridIterator {
    pub(crate) fn new(lat: Vec<f32>, lon: Vec<f32>, ij: GridPointIndexIterator) -> Self {
        Self { lat, lon, ij }
    }
}

impl Iterator for LatLonGridIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, j) = self.ij.next()?;
        Some((self.lat[j], self.lon[i]))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ij.size_hint()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ScanningMode(pub u8);

impl ScanningMode {
    /// Returns `true` if points of the first row or column scan in the `+i`
    /// (`+x`) direction.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::ScanningMode(0b00000000).scans_positively_for_i(),
    ///     true
    /// );
    /// ```
    pub fn scans_positively_for_i(&self) -> bool {
        self.0 & 0b10000000 == 0
    }

    /// Returns `true` if points of the first row or column scan in the `+j`
    /// (`+y`) direction.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::ScanningMode(0b00000000).scans_positively_for_j(),
    ///     false
    /// );
    /// ```
    pub fn scans_positively_for_j(&self) -> bool {
        self.0 & 0b01000000 != 0
    }

    /// Returns `true` if adjacent points in `i` (`x`) direction are
    /// consecutive.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(grib::ScanningMode(0b00000000).is_consecutive_for_i(), true);
    /// ```
    pub fn is_consecutive_for_i(&self) -> bool {
        self.0 & 0b00100000 == 0
    }

    /// Returns `true` if adjacent rows scans in the opposite direction.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::ScanningMode(0b00000000).scans_alternating_rows(),
    ///     false
    /// );
    /// ```
    pub fn scans_alternating_rows(&self) -> bool {
        self.0 & 0b00010000 != 0
    }

    pub(crate) fn has_unsupported_flags(&self) -> bool {
        self.0 & 0b00001111 != 0
    }
}

mod earth;

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use super::*;

    macro_rules! assert_almost_eq {
        ($a1:expr, $a2:expr, $d:expr) => {
            if !($a1 - $a2 < $d || $a2 - $a1 < $d) {
                panic!();
            }
        };
    }

    fn assert_coord_almost_eq((x1, y1): (f32, f32), (x2, y2): (f32, f32), delta: f32) {
        assert_almost_eq!(x1, x2, delta);
        assert_almost_eq!(y1, y2, delta);
    }

    #[test]
    fn test_lat_lon_grid_definition_and_iteration() {
        let grid = LatLonGridDefinition {
            ni: 3,
            nj: 2,
            first_point_lat: 36_000_000,
            first_point_lon: 135_000_000,
            last_point_lat: 35_000_000,
            last_point_lon: 137_000_000,
            scanning_mode: ScanningMode(0b00000000),
        };
        let actual = grid.latlons().unwrap().collect::<Vec<_>>();
        let expected = vec![
            (36., 135.),
            (36., 136.),
            (36., 137.),
            (35., 135.),
            (35., 136.),
            (35., 137.),
        ];
        assert_eq!(actual, expected)
    }

    macro_rules! test_consistencies_between_lat_lon_and_scanning_mode {
        ($((
            $name:ident,
            $first_point_lat:expr,
            $first_point_lon:expr,
            $last_point_lat:expr,
            $last_point_lon:expr,
            $scanning_mode:expr,
            $expected:expr
        ),)*) => ($(
            #[test]
            fn $name() {
                let grid = LatLonGridDefinition {
                    ni: 1,
                    nj: 1,
                    first_point_lat: $first_point_lat,
                    first_point_lon: $first_point_lon,
                    last_point_lat: $last_point_lat,
                    last_point_lon: $last_point_lon,
                    scanning_mode: ScanningMode($scanning_mode),
                };
                assert_eq!(grid.is_consistent(), $expected)
            }
        )*);
    }

    test_consistencies_between_lat_lon_and_scanning_mode! {
        (
            consistency_between_lat_decrease_and_scanning_mode_0b00000000,
            37_000_000,
            140_000_000,
            36_000_000,
            141_000_000,
            0b00000000,
            true
        ),
        (
            consistency_between_lat_decrease_and_scanning_mode_0b01000000,
            37_000_000,
            140_000_000,
            36_000_000,
            141_000_000,
            0b01000000,
            false
        ),
        (
            consistency_between_lat_increase_and_scanning_mode_0b00000000,
            36_000_000,
            140_000_000,
            37_000_000,
            141_000_000,
            0b00000000,
            false
        ),
        (
            consistency_between_lat_increase_and_scanning_mode_0b01000000,
            36_000_000,
            140_000_000,
            37_000_000,
            141_000_000,
            0b01000000,
            true
        ),
        (
            consistency_between_lon_increase_and_scanning_mode_0b00000000,
            37_000_000,
            140_000_000,
            36_000_000,
            141_000_000,
            0b00000000,
            true
        ),
        (
            consistency_between_lon_increase_and_scanning_mode_0b10000000,
            37_000_000,
            140_000_000,
            36_000_000,
            141_000_000,
            0b10000000,
            false
        ),
        (
            consistency_between_lon_decrease_and_scanning_mode_0b00000000,
            37_000_000,
            141_000_000,
            36_000_000,
            140_000_000,
            0b00000000,
            false
        ),
        (
            consistency_between_lon_decrease_and_scanning_mode_0b10000000,
            37_000_000,
            141_000_000,
            36_000_000,
            140_000_000,
            0b10000000,
            true
        ),
    }

    macro_rules! test_lat_lon_grid_iter {
        ($(($name:ident, $scanning_mode:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let lat = (0..3).into_iter().map(|i| i as f32).collect::<Vec<_>>();
                let lon = (10..12).into_iter().map(|i| i as f32).collect::<Vec<_>>();
                let scanning_mode = ScanningMode($scanning_mode);
                let ij= GridPointIndexIterator::new(lon.len(), lat.len(), scanning_mode);
                let actual = LatLonGridIterator::new(lat, lon, ij).collect::<Vec<_>>();
                assert_eq!(actual, $expected);
            }
        )*);
    }

    test_lat_lon_grid_iter! {
        (
            lat_lon_grid_iter_with_scanning_mode_0b00000000,
            0b00000000,
            vec![
                (0., 10.),
                (0., 11.),
                (1., 10.),
                (1., 11.),
                (2., 10.),
                (2., 11.),
            ]
        ),
        (
            lat_lon_grid_iter_with_scanning_mode_0b00100000,
            0b00100000,
            vec![
                (0., 10.),
                (1., 10.),
                (2., 10.),
                (0., 11.),
                (1., 11.),
                (2., 11.),
            ]
        ),
        (
            lat_lon_grid_iter_with_scanning_mode_0b00010000,
            0b00010000,
            vec![
                (0., 10.),
                (0., 11.),
                (1., 11.),
                (1., 10.),
                (2., 10.),
                (2., 11.),
            ]
        ),
        (
            lat_lon_grid_iter_with_scanning_mode_0b00110000,
            0b00110000,
            vec![
                (0., 10.),
                (1., 10.),
                (2., 10.),
                (2., 11.),
                (1., 11.),
                (0., 11.),
            ]
        ),
    }

    #[test]
    fn lat_lon_grid_iterator_size_hint() {
        let lat = (0..3).into_iter().map(|i| i as f32).collect::<Vec<_>>();
        let lon = (10..12).into_iter().map(|i| i as f32).collect::<Vec<_>>();
        let scanning_mode = ScanningMode(0b00000000);
        let ij = GridPointIndexIterator::new(lon.len(), lat.len(), scanning_mode);
        let mut iter = LatLonGridIterator::new(lat, lon, ij);

        assert_eq!(iter.size_hint(), (6, Some(6)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (5, Some(5)));
    }

    #[test]
    fn lambert_grid_definition_from_buf() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();

        let f = std::fs::File::open("testdata/ds.critfireo.bin.xz")?;
        let f = BufReader::new(f);
        let mut f = xz2::bufread::XzDecoder::new(f);
        f.read_to_end(&mut buf)?;

        let actual = LambertGridDefinition::from_buf(&buf[0x83..]);
        let expected = LambertGridDefinition {
            earth_shape: EarthShapeDefinition {
                shape_of_the_earth: 1,
                scale_factor_of_radius_of_spherical_earth: 0,
                scaled_value_of_radius_of_spherical_earth: 6371200,
                scale_factor_of_earth_major_axis: 0,
                scaled_value_of_earth_major_axis: 0,
                scale_factor_of_earth_minor_axis: 0,
                scaled_value_of_earth_minor_axis: 0,
            },
            ni: 2145,
            nj: 1377,
            first_point_lat: 20190000,
            first_point_lon: 238449996,
            lad: 25000000,
            lov: 265000000,
            dx: 2539703,
            dy: 2539703,
            scanning_mode: ScanningMode(0b01010000),
            latin1: 25000000,
            latin2: 25000000,
        };
        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn lambert_grid_latlon_computation() -> Result<(), Box<dyn std::error::Error>> {
        let grid_def = LambertGridDefinition {
            earth_shape: EarthShapeDefinition {
                shape_of_the_earth: 1,
                scale_factor_of_radius_of_spherical_earth: 0,
                scaled_value_of_radius_of_spherical_earth: 6371200,
                scale_factor_of_earth_major_axis: 0,
                scaled_value_of_earth_major_axis: 0,
                scale_factor_of_earth_minor_axis: 0,
                scaled_value_of_earth_minor_axis: 0,
            },
            ni: 2145,
            nj: 1377,
            first_point_lat: 20190000,
            first_point_lon: 238449996,
            lad: 25000000,
            lov: 265000000,
            dx: 2539703,
            dy: 2539703,
            scanning_mode: ScanningMode(0b01010000),
            latin1: 25000000,
            latin2: 25000000,
        };
        let latlons = grid_def.latlons()?.collect::<Vec<_>>();
        let delta = 1e-10;
        assert_coord_almost_eq(latlons[0], (20.19, -121.550004), delta);
        assert_coord_almost_eq(latlons[1], (20.19442682, -121.52621665), delta);
        assert_coord_almost_eq(
            latlons[latlons.len() - 2],
            (50.10756403, -60.91298217),
            delta,
        );
        assert_coord_almost_eq(
            latlons[latlons.len() - 1],
            (50.1024611, -60.88202274),
            delta,
        );

        Ok(())
    }
}
