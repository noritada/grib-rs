use super::{
    GridPointIndexIterator, ScanningMode,
    helpers::{RegularGridIterator, evenly_spaced_degrees, evenly_spaced_longitudes},
};
use crate::{
    error::GribError,
    helpers::{GribInt, read_as},
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

    /// Returns an iterator over latitudes and longitudes of grid points in
    /// degrees.
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
    pub fn latlons(&self) -> Result<RegularGridIterator, GribError> {
        if !self.is_consistent_for_j() {
            return Err(GribError::InvalidValueError(
                "Latitudes for first/last grid points are not consistent with scanning mode"
                    .to_owned(),
            ));
        }

        let ij = self.ij()?;
        let lat = evenly_spaced_degrees(
            self.first_point_lat as f32,
            self.last_point_lat as f32,
            (self.nj - 1) as usize,
        );
        let lon = evenly_spaced_longitudes(
            self.first_point_lon,
            self.last_point_lon,
            (self.ni - 1) as usize,
            self.scanning_mode,
        );

        let iter = RegularGridIterator::new(lat, lon, ij);
        Ok(iter)
    }

    pub(crate) fn is_consistent_for_j(&self) -> bool {
        let lat_diff = self.last_point_lat - self.first_point_lat;
        !((lat_diff > 0) ^ self.scanning_mode.scans_positively_for_j())
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_lat_lon_calculation_for_inconsistent_longitude_definitions {
        ($((
            $name:ident,
            $grid:expr,
            $expected_head:expr,
            $expected_tail:expr
        ),)*) => ($(
            #[test]
            fn $name() {
                let grid = $grid;
                let latlons = grid.latlons();
                assert!(latlons.is_ok());

                let latlons = latlons.unwrap();
                let actual = latlons.clone().take(3).collect::<Vec<_>>();
                let expected = $expected_head;
                assert_eq!(actual, expected);

                let (len, _) = latlons.size_hint();
                let actual = latlons.skip(len - 3).collect::<Vec<_>>();
                let expected = $expected_tail;
                assert_eq!(actual, expected);
            }
        )*);
    }

    test_lat_lon_calculation_for_inconsistent_longitude_definitions! {
        (
            lat_lon_calculation_for_increasing_longitudes_and_positive_direction_scan,
            LatLonGridDefinition {
                ni: 1500,
                nj: 751,
                first_point_lat: -90000000,
                first_point_lon: 0,
                last_point_lat: 90000000,
                last_point_lon: 359760000,
                scanning_mode: ScanningMode(0b01000000),
            },
            vec![(-90.0, 0.0), (-90.0, 0.24), (-90.0, 0.48)],
            vec![(90.0, 359.28), (90.0, 359.52), (90.0, 359.76)]
        ),
        (
            // grid point definition extracted from
            // testdata/CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2
            lat_lon_calculation_for_decreasing_longitudes_and_positive_direction_scan,
            LatLonGridDefinition {
                ni: 1500,
                nj: 751,
                first_point_lat: -90000000,
                first_point_lon: 180000000,
                last_point_lat: 90000000,
                last_point_lon: 179760000,
                scanning_mode: ScanningMode(0b01000000),
            },
            vec![(-90.0, 180.0), (-90.0, 180.24), (-90.0, 180.48)],
            vec![(90.0, 179.28003), (90.0, 179.52002), (90.0, 179.76001)]
        ),
        (
            lat_lon_calculation_for_decreasing_longitudes_and_negative_direction_scan,
            LatLonGridDefinition {
                ni: 1500,
                nj: 751,
                first_point_lat: -90000000,
                first_point_lon: 359760000,
                last_point_lat: 90000000,
                last_point_lon: 0,
                scanning_mode: ScanningMode(0b11000000),
            },
            vec![(-90.0, 359.76), (-90.0, 359.52), (-90.0, 359.28)],
            vec![(90.0, 0.48), (90.0, 0.24), (90.0, 0.0)]
        ),
        (
            lat_lon_calculation_for_increasing_longitudes_and_negative_direction_scan,
            LatLonGridDefinition {
                ni: 1500,
                nj: 751,
                first_point_lat: -90000000,
                first_point_lon: 179760000,
                last_point_lat: 90000000,
                last_point_lon: 180000000,
                scanning_mode: ScanningMode(0b11000000),
            },
            vec![(-90.0, 179.76001), (-90.0, 179.52002), (-90.0, 179.28003)],
            vec![(90.0, 180.48), (90.0, 180.24), (90.0, 180.0)]
        ),
    }

    macro_rules! test_consistencies_between_lat_lon_and_scanning_mode {
        ($((
            $name:ident,
            $first_point_lat:expr,
            $first_point_lon:expr,
            $last_point_lat:expr,
            $last_point_lon:expr,
            $scanning_mode:expr,
            $expected_for_j:expr
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
                assert_eq!(grid.is_consistent_for_j(), $expected_for_j);
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
    }
}
