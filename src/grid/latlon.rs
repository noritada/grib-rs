use super::{GridPointIndexIterator, ScanningMode};
use crate::{
    error::GribError,
    utils::{read_as, GribInt},
};

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
    /// Returns the shape of the grid, i.e. a tuple of the number of grids in
    /// the i and j directions.
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
    /// let shape = def.grid_shape();
    /// assert_eq!(shape, (2, 3));
    /// ```
    pub fn grid_shape(&self) -> (usize, usize) {
        (self.ni as usize, self.nj as usize)
    }

    /// Returns the grid type.
    pub fn short_name(&self) -> &'static str {
        "regular_ll"
    }

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

    pub(crate) fn is_consistent(&self) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let lat = (0..3).map(|i| i as f32).collect::<Vec<_>>();
        let lon = (10..12).map(|i| i as f32).collect::<Vec<_>>();
        let scanning_mode = ScanningMode(0b00000000);
        let ij = GridPointIndexIterator::new(lon.len(), lat.len(), scanning_mode);
        let mut iter = LatLonGridIterator::new(lat, lon, ij);

        assert_eq!(iter.size_hint(), (6, Some(6)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (5, Some(5)));
    }
}
