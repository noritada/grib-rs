#[cfg(feature = "gridpoints-proj")]
use proj::Proj;

use super::ScanningMode;
#[allow(unused_imports)]
use crate::GribError;
use crate::GridPointIndexIterator;

pub(crate) fn evenly_spaced_longitudes(
    start_microdegree: i32,
    end_microdegree: i32,
    div: usize,
    scanning_mode: ScanningMode,
) -> Vec<f32> {
    let diff = end_microdegree - start_microdegree;
    let is_consistent = !((diff > 0) ^ scanning_mode.scans_positively_for_i());

    let (start, end) = (start_microdegree as f32, end_microdegree as f32);
    let (start, end) = if is_consistent {
        (start, end)
    } else if start_microdegree > end_microdegree {
        (start, end + 360_000_000_f32)
    } else {
        (start + 360_000_000_f32, end)
    };

    let lons = evenly_spaced_degrees(start, end, div);

    if is_consistent {
        lons
    } else {
        lons.into_iter()
            .map(|x| if x < 360.0 { x } else { x - 360.0 })
            .collect()
    }
}

pub(crate) fn evenly_spaced_degrees(
    start_microdegree: f32,
    end_microdegree: f32,
    div: usize,
) -> Vec<f32> {
    let delta = (end_microdegree - start_microdegree) / div as f32;
    (0..=div)
        .map(move |x| (start_microdegree + x as f32 * delta) / 1_000_000_f32)
        .collect()
}

/// An iterator over latitudes and longitudes of grid points of a regular grid.
#[derive(Clone)]
pub struct RegularGridIterator {
    lat: Vec<f32>,
    lon: Vec<f32>,
    ij: GridPointIndexIterator,
}

impl RegularGridIterator {
    pub(crate) fn new(lat: Vec<f32>, lon: Vec<f32>, ij: GridPointIndexIterator) -> Self {
        Self { lat, lon, ij }
    }
}

impl Iterator for RegularGridIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        let (i, j) = self.ij.next()?;
        Some((self.lat[j], self.lon[i]))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.ij.size_hint()
    }
}

#[cfg(feature = "gridpoints-proj")]
pub(crate) fn latlons_from_projection_definition_and_first_point(
    proj_def: &str,
    first_point_latlon_in_degrees: (f64, f64),
    delta_in_meters: (f64, f64),
    indices: GridPointIndexIterator,
) -> Result<std::vec::IntoIter<(f32, f32)>, GribError> {
    let projection = Proj::new(proj_def).map_err(|e| GribError::Unknown(e.to_string()))?;
    let (first_point_lat, first_point_lon) = first_point_latlon_in_degrees;
    let (first_corner_x, first_corner_y) = projection
        .project(
            (first_point_lon.to_radians(), first_point_lat.to_radians()),
            false,
        )
        .map_err(|e| GribError::Unknown(e.to_string()))?;

    let (dx, dy) = delta_in_meters;
    let mut xy = indices
        .map(|(i, j)| {
            (
                first_corner_x + dx * i as f64,
                first_corner_y + dy * j as f64,
            )
        })
        .collect::<Vec<_>>();

    let lonlat = projection
        .project_array(&mut xy, true)
        .map_err(|e| GribError::Unknown(e.to_string()))?;
    let latlon = lonlat
        .iter_mut()
        .map(|(lon, lat)| (lat.to_degrees() as f32, lon.to_degrees() as f32))
        .collect::<Vec<_>>();

    Ok(latlon.into_iter())
}

#[cfg(test)]
pub(crate) mod test_helpers {
    macro_rules! assert_almost_eq {
        ($a1:expr, $a2:expr, $d:expr) => {
            if $a1 - $a2 > $d {
                panic!(
                    "assertion a1 - a2 <= delta failed\n a1 - a2: {} - {}\n   delta: {}",
                    $a1, $a2, $d
                );
            } else if $a2 - $a1 > $d {
                panic!(
                    "assertion a2 - a1 <= delta failed\n a2 - a1: {} - {}\n   delta: {}",
                    $a2, $a1, $d
                );
            }
        };
    }
    pub(crate) use assert_almost_eq;

    macro_rules! test_assert_almost_eq_do_not_panic {
        ($((
            $name:ident,
            $a1:expr,
            $a2:expr,
            $d:expr
        ),)*) => ($(
            #[test]
            fn $name() {
                assert_almost_eq!($a1, $a2, $d)
            }
        )*);
    }

    test_assert_almost_eq_do_not_panic! {
        (assert_almost_eq_does_not_panic_for_positive_lt_positive, 1.01, 1.02, 0.1),
        (assert_almost_eq_does_not_panic_for_positive_gt_positive, 1.02, 1.01, 0.1),
        (assert_almost_eq_does_not_panic_for_negative_lt_negative, -1.02, -1.01, 0.1),
        (assert_almost_eq_does_not_panic_for_negative_gt_negative, -1.01, -1.02, 0.1),
        (assert_almost_eq_does_not_panic_for_positive_negative, 0.01, -0.01, 0.1),
        (assert_almost_eq_does_not_panic_for_negative_positive, -0.01, 0.01, 0.1),
    }

    macro_rules! test_assert_almost_eq_panic {
        ($((
            $name:ident,
            $a1:expr,
            $a2:expr,
            $d:expr,
            $message:expr
        ),)*) => ($(
            #[test]
            #[should_panic(expected = $message)]
            fn $name() {
                assert_almost_eq!($a1, $a2, $d)
            }
        )*);
    }

    test_assert_almost_eq_panic! {
        (
            assert_almost_eq_panics_for_positive_lt_positive, 1.01, 1.02, 0.001,
            " a2 - a1: 1.02 - 1.01\n   delta: 0.001"
        ),
        (
            assert_almost_eq_panics_for_positive_gt_positive, 1.02, 1.01, 0.001,
            " a1 - a2: 1.02 - 1.01\n   delta: 0.001"
        ),
        (
            assert_almost_eq_panics_for_negative_lt_negative, -1.02, -1.01, 0.001,
            " a2 - a1: -1.01 - -1.02\n   delta: 0.001"
        ),
        (
            assert_almost_eq_panics_for_negative_gt_negative, -1.01, -1.02, 0.001,
            " a1 - a2: -1.01 - -1.02\n   delta: 0.001"
        ),
        (
            assert_almost_eq_panics_for_positive_negative, 0.01, -0.01, 0.001,
            " a1 - a2: 0.01 - -0.01\n   delta: 0.001"
        ),
        (
            assert_almost_eq_panics_for_negative_positive, -0.01, 0.01, 0.001,
            " a2 - a1: 0.01 - -0.01\n   delta: 0.001"
        ),
        (
            assert_almost_eq_panic_message_containing_trailing_zeros, -0.0100, 0.0100, 0.0010,
            " a2 - a1: 0.01 - -0.01\n   delta: 0.001"
        ),
    }

    #[allow(dead_code)]
    pub(crate) fn assert_coord_almost_eq((x1, y1): (f32, f32), (x2, y2): (f32, f32), delta: f32) {
        assert_almost_eq!(x1, x2, delta);
        assert_almost_eq!(y1, y2, delta);
    }
}

#[cfg(test)]
mod tests {
    use super::{super::ScanningMode, *};

    macro_rules! test_lat_lon_grid_iter {
        ($(($name:ident, $scanning_mode:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let lat = (0..3).into_iter().map(|i| i as f32).collect::<Vec<_>>();
                let lon = (10..12).into_iter().map(|i| i as f32).collect::<Vec<_>>();
                let scanning_mode = ScanningMode($scanning_mode);
                let ij= GridPointIndexIterator::new(lon.len(), lat.len(), scanning_mode);
                let actual = RegularGridIterator::new(lat, lon, ij).collect::<Vec<_>>();
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
        let mut iter = RegularGridIterator::new(lat, lon, ij);

        assert_eq!(iter.size_hint(), (6, Some(6)));
        let _ = iter.next();
        assert_eq!(iter.size_hint(), (5, Some(5)));
    }
}
