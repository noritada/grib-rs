use crate::error::GribError;

pub(crate) struct LatLonGridDefinition {
    ni: u32,
    nj: u32,
    first_point_lat: i32,
    first_point_lon: i32,
    last_point_lat: i32,
    last_point_lon: i32,
    scanning_mode: ScanningMode,
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

    pub(crate) fn iter(&self) -> Result<LatLonGridIterator, GribError> {
        if !self.is_consistent() {
            return Err(GribError::InvalidValueError("Latitude and longitude for first/last grid points are not consistent with scanning mode".to_owned()));
        }

        let lat_diff = self.last_point_lat - self.first_point_lat;
        let lon_diff = self.last_point_lon - self.first_point_lon;
        let (num_div_lat, num_div_lon) = ((self.nj - 1) as i32, (self.ni - 1) as i32);
        let lat_delta = lat_diff as f32 / num_div_lat as f32;
        let lat = (0..=num_div_lat)
            .into_iter()
            .map(|x| (self.first_point_lat as f32 + x as f32 * lat_delta) / 1_000_000 as f32)
            .collect();
        let lon_delta = lon_diff as f32 / num_div_lon as f32;
        let lon = (0..=num_div_lon)
            .into_iter()
            .map(|x| (self.first_point_lon as f32 + x as f32 * lon_delta) / 1_000_000 as f32)
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
}

pub(crate) struct LatLonGridIterator {
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

#[derive(Clone, Copy)]
pub(crate) struct ScanningMode(u8);

impl ScanningMode {
    pub(crate) fn scans_positively_for_i(&self) -> bool {
        self.0 & 0b10000000 == 0
    }

    pub(crate) fn scans_positively_for_j(&self) -> bool {
        self.0 & 0b01000000 != 0
    }

    pub(crate) fn is_consecutive_for_i(&self) -> bool {
        self.0 & 0b00100000 == 0
    }

    pub(crate) fn scans_alternating_rows(&self) -> bool {
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
        let actual = grid.iter().unwrap().collect::<Vec<_>>();
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
