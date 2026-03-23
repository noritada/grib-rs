use super::helpers::{RegularGridIterator, evenly_spaced_degrees, evenly_spaced_longitudes};
use crate::{GridPointIndex, LatLons, def::grib2::template::param_set, error::GribError};

impl crate::GridShortName for param_set::LatLonGrid {
    fn short_name(&self) -> &'static str {
        "regular_ll"
    }
}

impl GridPointIndex for param_set::LatLonGrid {
    fn grid_shape(&self) -> (usize, usize) {
        (self.grid.ni as usize, self.grid.nj as usize)
    }

    fn scanning_mode(&self) -> &param_set::ScanningMode {
        &self.scanning_mode
    }
}

impl LatLons for param_set::LatLonGrid {
    type Iter<'a> = RegularGridIterator;

    fn latlons_unchecked<'a>(&'a self) -> Result<Self::Iter<'a>, GribError> {
        if !self.is_consistent_for_j() {
            return Err(GribError::InvalidValueError(
                "Latitudes for first/last grid points are not consistent with scanning mode"
                    .to_owned(),
            ));
        }

        let ij = self.ij()?;
        let lat = evenly_spaced_degrees(
            self.grid.first_point_lat as f32,
            self.grid.last_point_lat as f32,
            (self.grid.nj - 1) as usize,
        );
        let lon = evenly_spaced_longitudes(
            self.grid.first_point_lon,
            self.grid.last_point_lon,
            (self.grid.ni - 1) as usize,
            self.scanning_mode,
        );

        let iter = RegularGridIterator::new(lat, lon, ij);
        Ok(iter)
    }
}

impl param_set::LatLonGrid {
    pub(crate) fn is_consistent_for_j(&self) -> bool {
        let lat_diff = self.grid.last_point_lat - self.grid.first_point_lat;
        !((lat_diff > 0) ^ self.scanning_mode.scans_positively_for_j())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::helpers::test_helpers::assert_coord_almost_eq;

    macro_rules! test_lat_lon_calculation_for_inconsistent_longitude_definitions {
        ($((
            $name:ident,
            $grid:expr,
            $scanning_mode:expr,
            $expected_head:expr,
            $expected_tail:expr
        ),)*) => ($(
            #[test]
            fn $name() {
                let grid = param_set::LatLonGrid {
                    grid: $grid,
                    i_direction_inc: 0xffffffff,
                    j_direction_inc: 0xffffffff,
                    scanning_mode: $scanning_mode,
                };
                let latlons = grid.latlons();
                assert!(latlons.is_ok());

                let delta = 1e-4;

                let latlons = latlons.unwrap();
                let actual = latlons.clone().take(3).collect::<Vec<_>>();
                let expected = $expected_head;
                for (a, e) in actual.iter().zip(expected) {
                    assert_coord_almost_eq(*a, e, delta);
                }

                let (len, _) = latlons.size_hint();
                let actual = latlons.skip(len - 3).collect::<Vec<_>>();
                let expected = $expected_tail;
                for (a, e) in actual.iter().zip(expected) {
                    assert_coord_almost_eq(*a, e, delta);
                }
            }
        )*);
    }

    test_lat_lon_calculation_for_inconsistent_longitude_definitions! {
        (
            lat_lon_calculation_for_increasing_longitudes_and_positive_direction_scan,
            param_set::Grid {
                ni: 1500,
                nj: 751,
                initial_production_domain_basic_angle: 0,
                basic_angle_subdivisions: 0xffffffff,
                first_point_lat: -90000000,
                first_point_lon: 0,
                resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00110000),
                last_point_lat: 90000000,
                last_point_lon: 359760000,
            },
            param_set::ScanningMode(0b01000000),
            vec![(-90.0, 0.0), (-90.0, 0.24), (-90.0, 0.48)],
            vec![(90.0, -0.72), (90.0, -0.48), (90.0, -0.24)]
        ),
        (
            // grid point definition extracted from
            // testdata/CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2
            lat_lon_calculation_for_decreasing_longitudes_and_positive_direction_scan,
            param_set::Grid {
                ni: 1500,
                nj: 751,
                initial_production_domain_basic_angle: 0,
                basic_angle_subdivisions: 0xffffffff,
                first_point_lat: -90000000,
                first_point_lon: 180000000,
                resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00110000),
                last_point_lat: 90000000,
                last_point_lon: 179760000,
            },
            param_set::ScanningMode(0b01000000),
            vec![(-90.0, -180.0), (-90.0, -179.76), (-90.0, -179.52)],
            vec![(90.0, 179.28003), (90.0, 179.52002), (90.0, 179.76001)]
        ),
        (
            lat_lon_calculation_for_decreasing_longitudes_and_negative_direction_scan,
            param_set::Grid {
                ni: 1500,
                nj: 751,
                initial_production_domain_basic_angle: 0,
                basic_angle_subdivisions: 0xffffffff,
                first_point_lat: -90000000,
                first_point_lon: 359760000,
                resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00110000),
                last_point_lat: 90000000,
                last_point_lon: 0,
            },
            param_set::ScanningMode(0b11000000),
            vec![(-90.0, -0.24), (-90.0, -0.48), (-90.0, -0.72)],
            vec![(90.0, 0.48), (90.0, 0.24), (90.0, 0.0)]
        ),
        (
            lat_lon_calculation_for_increasing_longitudes_and_negative_direction_scan,
            param_set::Grid {
                ni: 1500,
                nj: 751,
                initial_production_domain_basic_angle: 0,
                basic_angle_subdivisions: 0xffffffff,
                first_point_lat: -90000000,
                first_point_lon: 179760000,
                resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00110000),
                last_point_lat: 90000000,
                last_point_lon: 180000000,
            },
            param_set::ScanningMode(0b11000000),
            vec![(-90.0, 179.76001), (-90.0, 179.52002), (-90.0, 179.28003)],
            vec![(90.0, -179.52), (90.0, -179.76), (90.0, -180.0)]
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
                let grid = param_set::LatLonGrid {
                    grid: param_set::Grid {
                        ni: 1,
                        nj: 1,
                        initial_production_domain_basic_angle: 0,
                        basic_angle_subdivisions: 0xffffffff,
                        first_point_lat: $first_point_lat,
                        first_point_lon: $first_point_lon,
                        resolution_and_component_flags:
                            param_set::ResolutionAndComponentFlags(0b00110000),
                        last_point_lat: $last_point_lat,
                        last_point_lon: $last_point_lon,
                    },
                    i_direction_inc: 0xffffffff,
                    j_direction_inc: 0xffffffff,
                    scanning_mode: param_set::ScanningMode($scanning_mode),
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
