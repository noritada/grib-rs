use crate::def::grib2::template::param_set;

/// An auxiliary struct for creating the definition of the latitude/longitude
/// coordinate system
/// ([`LatLonGrid`](`crate::def::grib2::template::param_set::LatLonGrid`)) in a
/// more intuitive way.
pub struct LatLonGrid {
    /// Shape of the grid, in the form `(ni, nj)`.
    pub shape: (usize, usize),
    /// First coordinate point, in the form `(lat, lon)`. The order in which you
    /// specify the "first" point should correspond to the order of the array of
    /// point values.
    pub first_point: (f64, f64),
    /// The coordinate point diagonally opposite the first coordinate point, in
    /// the form `(lat, lon)`.
    pub last_point: (f64, f64),
    /// Whether the grid point values consecutive in the `i`-direction
    /// (longitude direction) or not.
    pub i_consecutive: bool,
}

impl LatLonGrid {
    /// Returns the number of points of the grid.
    pub fn num_points(&self) -> usize {
        self.shape.0 * self.shape.1
    }

    /// Returns the scanning mode of the grid.
    pub fn scanning_mode(&self) -> param_set::ScanningMode {
        let mut scan_mode: u8 = 0b00000000;
        if self.first_point.1 > self.last_point.1 {
            scan_mode |= 0b10000000;
        }
        if self.first_point.0 < self.last_point.0 {
            scan_mode |= 0b01000000;
        }
        if !self.i_consecutive {
            scan_mode |= 0b00100000;
        }
        param_set::ScanningMode(scan_mode)
    }
}

impl From<&LatLonGrid> for param_set::LatLonGrid {
    fn from(value: &LatLonGrid) -> Self {
        let (ni, nj) = (value.shape.0 as u32, value.shape.1 as u32);

        let (first_point_lat, first_point_lon) = (
            microdegrees(value.first_point.0),
            microdegrees(normalized_longitude(value.first_point.1)) as u32,
        );
        let (last_point_lat, last_point_lon) = (
            microdegrees(value.last_point.0),
            microdegrees(normalized_longitude(value.last_point.1)) as u32,
        );

        let i_direction_inc =
            inc_in_microdegrees(value.first_point.1, value.last_point.1, value.shape.0 - 1);
        let j_direction_inc =
            inc_in_microdegrees(value.first_point.0, value.last_point.0, value.shape.1 - 1);
        let scanning_mode = value.scanning_mode();

        param_set::LatLonGrid {
            grid: param_set::Grid {
                ni,
                nj,
                initial_production_domain_basic_angle: 0,
                basic_angle_subdivisions: 0xffffffff,
                first_point_lat,
                first_point_lon,
                resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00110000),
                last_point_lat,
                last_point_lon,
            },
            i_direction_inc,
            j_direction_inc,
            scanning_mode,
        }
    }
}

fn inc_in_microdegrees(first: f64, last: f64, n_spacing: usize) -> u32 {
    let diff = if first < last {
        last - first
    } else {
        first - last
    };
    microdegrees(diff) as u32 / n_spacing as u32
}

fn microdegrees(val_in_degrees: f64) -> i32 {
    (val_in_degrees * 1e6) as i32
}

fn normalized_longitude(val: f64) -> f64 {
    if val < 0. { val + 360. } else { val }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_lat_lon_grid_definition {
        ($(($name:ident, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let input = $input;
                let params = crate::def::grib2::template::param_set::LatLonGrid::from(&input);
                let actual = (
                    (params.grid.ni, params.grid.nj),
                    (params.grid.first_point_lat, params.grid.first_point_lon),
                    (params.grid.last_point_lat, params.grid.last_point_lon),
                    (params.i_direction_inc, params.j_direction_inc),
                    params.scanning_mode,
                );
                assert_eq!(actual, $expected);
            }
        )*);
    }

    test_lat_lon_grid_definition! {
        (
            lat_lon_grid_definition_for_neg_lat_inc_and_pos_lon_inc,
            LatLonGrid {
                shape: (201, 101),
                first_point: (40., 130.),
                last_point: (30., 140.),
                i_consecutive: true,
            },
            (
                (201, 101),
                (40000000, 130000000),
                (30000000, 140000000),
                (50000, 100000),
                param_set::ScanningMode(0b00000000),
            )
        ),
        (
            lat_lon_grid_definition_for_neg_lat_inc_and_neg_lon_inc,
            LatLonGrid {
                shape: (201, 101),
                first_point: (40., 140.),
                last_point: (30., 130.),
                i_consecutive: true,
            },
            (
                (201, 101),
                (40000000, 140000000),
                (30000000, 130000000),
                (50000, 100000),
                param_set::ScanningMode(0b10000000),
            )
        ),
        (
            lat_lon_grid_definition_for_pos_lat_inc_and_pos_lon_inc,
            LatLonGrid {
                shape: (201, 101),
                first_point: (30., 130.),
                last_point: (40., 140.),
                i_consecutive: true,
            },
            (
                (201, 101),
                (30000000, 130000000),
                (40000000, 140000000),
                (50000, 100000),
                param_set::ScanningMode(0b01000000),
            )
        ),
        (
            lat_lon_grid_definition_for_pos_lat_inc_and_neg_lon_inc,
            LatLonGrid {
                shape: (201, 101),
                first_point: (30., 140.),
                last_point: (40., 130.),
                i_consecutive: true,
            },
            (
                (201, 101),
                (30000000, 140000000),
                (40000000, 130000000),
                (50000, 100000),
                param_set::ScanningMode(0b11000000),
            )
        ),
    }

    #[test]
    fn normalized_longitude_value() {
        assert_eq!(normalized_longitude(-360.), 0.);
        assert_eq!(normalized_longitude(-180.), 180.);
        assert_eq!(normalized_longitude(0.), 0.);
        assert_eq!(normalized_longitude(180.), 180.);
        assert_eq!(normalized_longitude(360.), 360.);
    }
}
