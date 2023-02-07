use crate::{
    error::GribError,
    utils::{read_as, GribInt},
};

#[derive(Clone)]
pub enum GridPointIterator {
    LatLon(LatLonGridIterator),
}

impl Iterator for GridPointIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::LatLon(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::LatLon(iter) => iter.size_hint(),
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
    pub(crate) fn new(
        ni: u32,
        nj: u32,
        first_point_lat: i32,
        first_point_lon: i32,
        last_point_lat: i32,
        last_point_lon: i32,
        scanning_mode: ScanningMode,
    ) -> Self {
        Self {
            ni,
            nj,
            first_point_lat,
            first_point_lon,
            last_point_lat,
            last_point_lon,
            scanning_mode,
        }
    }

    /// Returns an iterator over latitudes and longitudes of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn latlons(&self) -> Result<LatLonGridIterator, GribError> {
        if self.scanning_mode.has_unsupported_flags() {
            let ScanningMode(mode) = self.scanning_mode;
            return Err(GribError::NotSupported(format!("scanning mode {mode}")));
        }
        if !self.is_consistent() {
            return Err(GribError::InvalidValueError("Latitude and longitude for first/last grid points are not consistent with scanning mode".to_owned()));
        }

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

        let iter = LatLonGridIterator::new(lat, lon, self.scanning_mode);
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
        Self::new(
            ni,
            nj,
            first_point_lat,
            first_point_lon,
            last_point_lat,
            last_point_lon,
            ScanningMode(scanning_mode),
        )
    }
}

#[derive(Clone)]
pub struct LatLonGridIterator {
    major: Vec<f32>,
    minor: Vec<f32>,
    scanning_mode: ScanningMode,
    major_pos: usize,
    minor_pos: usize,
    increments: bool,
}

impl LatLonGridIterator {
    pub(crate) fn new(lat: Vec<f32>, lon: Vec<f32>, scanning_mode: ScanningMode) -> Self {
        let (major, minor) = if scanning_mode.is_consecutive_for_i() {
            (lat, lon)
        } else {
            (lon, lat)
        };

        Self {
            major,
            minor,
            scanning_mode,
            minor_pos: 0,
            major_pos: 0,
            increments: true,
        }
    }
}

impl Iterator for LatLonGridIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.major_pos == self.major.len() {
            return None;
        }

        let minor_pos = if self.increments {
            self.minor_pos
        } else {
            self.minor.len() - self.minor_pos - 1
        };
        let minor = self.minor[minor_pos];
        let major = self.major[self.major_pos];

        self.minor_pos += 1;
        if self.minor_pos == self.minor.len() {
            self.major_pos += 1;
            self.minor_pos = 0;
            if self.scanning_mode.scans_alternating_rows() {
                self.increments = !self.increments;
            }
        }

        if self.scanning_mode.is_consecutive_for_i() {
            Some((major, minor))
        } else {
            Some((minor, major))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.major.len() * self.minor.len();
        (len, Some(len))
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
    /// use grib::ScanningMode;
    ///
    /// let scanning_mode = ScanningMode(0b00000000);
    /// assert_eq!(scanning_mode.scans_positively_for_i(), true);
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
    /// use grib::ScanningMode;
    ///
    /// let scanning_mode = ScanningMode(0b00000000);
    /// assert_eq!(scanning_mode.scans_positively_for_j(), false);
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
    /// use grib::ScanningMode;
    ///
    /// let scanning_mode = ScanningMode(0b00000000);
    /// assert_eq!(scanning_mode.is_consecutive_for_i(), true);
    /// ```
    pub fn is_consecutive_for_i(&self) -> bool {
        self.0 & 0b00100000 == 0
    }

    /// Returns `true` if adjacent rows scans in the opposite direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use grib::ScanningMode;
    ///
    /// let scanning_mode = ScanningMode(0b00000000);
    /// assert_eq!(scanning_mode.scans_alternating_rows(), false);
    /// ```
    pub fn scans_alternating_rows(&self) -> bool {
        self.0 & 0b00010000 != 0
    }

    pub(crate) fn has_unsupported_flags(&self) -> bool {
        self.0 & 0b00001111 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lat_lon_grid_definition_and_iteration() {
        let grid = LatLonGridDefinition::new(
            3,
            2,
            36_000_000,
            135_000_000,
            35_000_000,
            137_000_000,
            ScanningMode(0b00000000),
        );
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
                let grid = LatLonGridDefinition::new(
                    1,
                    1,
                    $first_point_lat,
                    $first_point_lon,
                    $last_point_lat,
                    $last_point_lon,
                    ScanningMode($scanning_mode),
                );
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
                let lat = (0..3).into_iter().map(|i| i as f32).collect();
                let lon = (10..12).into_iter().map(|i| i as f32).collect();
                let scanning_mode = ScanningMode($scanning_mode);
                let actual = LatLonGridIterator::new(lat, lon, scanning_mode).collect::<Vec<_>>();
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
}
