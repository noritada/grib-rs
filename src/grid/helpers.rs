#[cfg(feature = "gridpoints-proj")]
use proj::Proj;

#[allow(unused_imports)]
use crate::{GribError, GridPointIndexIterator};

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
            if $a1 - $a2 > $d || $a2 - $a1 > $d {
                panic!();
            }
        };
    }

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
            $d:expr
        ),)*) => ($(
            #[test]
            #[should_panic]
            fn $name() {
                assert_almost_eq!($a1, $a2, $d)
            }
        )*);
    }

    test_assert_almost_eq_panic! {
        (assert_almost_eq_panics_for_positive_lt_positive, 1.01, 1.02, 0.001),
        (assert_almost_eq_panics_for_positive_gt_positive, 1.02, 1.01, 0.001),
        (assert_almost_eq_panics_for_negative_lt_negative, -1.02, -1.01, 0.001),
        (assert_almost_eq_panics_for_negative_gt_negative, -1.01, -1.02, 0.001),
        (assert_almost_eq_panics_for_positive_negative, 0.01, -0.01, 0.001),
        (assert_almost_eq_panics_for_negative_positive, -0.01, 0.01, 0.001),
    }

    #[allow(dead_code)]
    pub(crate) fn assert_coord_almost_eq((x1, y1): (f32, f32), (x2, y2): (f32, f32), delta: f32) {
        assert_almost_eq!(x1, x2, delta);
        assert_almost_eq!(y1, y2, delta);
    }
}
