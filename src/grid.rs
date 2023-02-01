pub(crate) struct LatLonGridIterator {
    major: Vec<f32>,
    minor: Vec<f32>,
    scanning_mode: ScanningMode,
    major_pos: usize,
    minor_pos: usize,
    increments: bool,
}

impl LatLonGridIterator {
    pub(crate) fn new(major: Vec<f32>, minor: Vec<f32>, scanning_mode: ScanningMode) -> Self {
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

    macro_rules! test_lat_lon_grid_iter {
        ($(($name:ident, $scanning_mode:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let major = (0..3).into_iter().map(|i| i as f32).collect();
                let minor = (10..12).into_iter().map(|i| i as f32).collect();
                let scanning_mode = ScanningMode($scanning_mode);
                let actual = LatLonGridIterator::new(major, minor, scanning_mode).collect::<Vec<_>>();
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
                (10., 0.),
                (11., 0.),
                (10., 1.),
                (11., 1.),
                (10., 2.),
                (11., 2.),
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
                (10., 0.),
                (11., 0.),
                (11., 1.),
                (10., 1.),
                (10., 2.),
                (11., 2.),
            ]
        ),
    }
}
