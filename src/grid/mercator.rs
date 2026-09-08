#[cfg(feature = "gridpoints-proj")]
use crate::projection::OsgeoProj;
#[cfg(not(feature = "gridpoints-proj"))]
use crate::projection::Project;
use crate::{
    GridPointIndex, LatLons,
    def::grib2::template::{Template3_10, param_set},
    error::GribError,
    grid::AngleUnit,
    projection,
};

impl crate::GridShortName for Template3_10 {
    fn short_name(&self) -> &'static str {
        "mercator"
    }
}

impl GridPointIndex for Template3_10 {
    fn grid_shape(&self) -> (usize, usize) {
        (self.ni as usize, self.nj as usize)
    }

    fn scanning_mode(&self) -> &param_set::ScanningMode {
        &self.scanning_mode
    }
}

impl LatLons for Template3_10 {
    type Iter<'a> = std::vec::IntoIter<(f32, f32)>;

    fn latlons_unchecked<'a>(&'a self) -> Result<Self::Iter<'a>, GribError> {
        if self.orientation != 0 {
            return Err(GribError::NotSupported(format!(
                "Mercator grid orientation {}",
                self.orientation
            )));
        }

        if !self.is_consistent_for_j() {
            return Err(GribError::InvalidValueError(
                "Latitudes for first/last grid points are not consistent with scanning mode"
                    .to_owned(),
            ));
        }

        let angle_units = self.angle_unit();
        let first_point_lon = self.first_point_lon as f64 * angle_units;
        let last_point_lon = self.last_point_lon as f64 * angle_units;
        let lon_diff = last_point_lon - first_point_lon;
        let (first_point_lon, last_point_lon) =
            if self.scanning_mode.scans_positively_for_i() && lon_diff < 0. {
                (first_point_lon, last_point_lon + 360.)
            } else if !self.scanning_mode.scans_positively_for_i() && lon_diff > 0. {
                (first_point_lon + 360., last_point_lon)
            } else {
                (first_point_lon, last_point_lon)
            };
        let first_point = (self.first_point_lat as f64 * angle_units, first_point_lon);
        let last_point = (self.last_point_lat as f64 * angle_units, last_point_lon);
        // ensure that all points are within the interval `[lon_0 - 180., lon_0
        // + 180.]` so that the delta can be calculated correctly.
        let lon_0 = (first_point_lon + last_point_lon) / 2.;

        let (a, b) = self.earth_shape.radii().ok_or_else(|| {
            GribError::NotSupported(format!(
                "unknown value of Code Table 3.2 (shape of the Earth): {}",
                self.earth_shape.shape
            ))
        })?;
        let params = projection::MercParams {
            ellipsoid: projection::Ellipsoid::from_a_and_b(a, b),
            lat_ts: self.lad as f64 * angle_units,
            lon_0,
        };

        #[cfg(feature = "gridpoints-proj")]
        {
            super::helpers::latlons_from_projection_with_first_point_and_last_point(
                &params.proj_args(),
                first_point,
                last_point,
                self.grid_shape(),
                self.ij()?,
            )
        }

        #[cfg(not(feature = "gridpoints-proj"))]
        {
            let projection = projection::Merc::new(&params)?;
            let (first_point_lat, first_point_lon) = first_point;
            let (first_corner_x, first_corner_y) = projection.project(
                &(first_point_lon.to_radians(), first_point_lat.to_radians()),
                false,
            )?;
            let (last_point_lat, last_point_lon) = last_point;
            let (last_corner_x, last_corner_y) = projection.project(
                &(last_point_lon.to_radians(), last_point_lat.to_radians()),
                false,
            )?;

            let dx = (last_corner_x - first_corner_x) / (self.ni - 1) as f64;
            let dy = (last_corner_y - first_corner_y) / (self.nj - 1) as f64;
            let latlons = self
                .ij()?
                .map(|(i, j)| {
                    projection
                        .project(
                            &(
                                first_corner_x + dx * i as f64,
                                first_corner_y + dy * j as f64,
                            ),
                            true,
                        )
                        .map(|(lon, lat)| (lat.to_degrees() as f32, lon.to_degrees() as f32))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(latlons.into_iter())
        }
    }
}

impl AngleUnit for Template3_10 {
    fn angle_unit(&self) -> f64 {
        1e-6
    }
}

impl Template3_10 {
    pub(crate) fn is_consistent_for_j(&self) -> bool {
        let lat_diff = self.last_point_lat - self.first_point_lat;
        !((lat_diff > 0) ^ self.scanning_mode.scans_positively_for_j())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::helpers::test_helpers::assert_coord_almost_eq;

    fn grid_definition() -> Template3_10 {
        // grid point definition extracted from testdata/ds.wwa.bin.xz
        Template3_10 {
            earth_shape: param_set::EarthShape {
                shape: 1,
                spherical_earth_radius: param_set::ScaledValue {
                    scale_factor: 0,
                    scaled_value: 6371200,
                },
                major_axis: param_set::ScaledValue {
                    scale_factor: 0,
                    scaled_value: 0,
                },
                minor_axis: param_set::ScaledValue {
                    scale_factor: 0,
                    scaled_value: 0,
                },
            },
            ni: 2517,
            nj: 1793,
            first_point_lat: -30419200,
            first_point_lon: 129906005,
            resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00000000),
            lad: 20000000,
            last_point_lat: 80010000,
            last_point_lon: 10710000,
            scanning_mode: param_set::ScanningMode(0b01010000),
            orientation: 0,
            di: 10000000,
            dj: 10000000,
        }
    }

    #[test]
    fn mercator_grid_latlon_computation() -> Result<(), Box<dyn std::error::Error>> {
        let grid_def = grid_definition();
        let latlons = grid_def.latlons()?.collect::<Vec<_>>();

        // Following lat/lon values are taken from the calculation results using
        // pygrib.
        let num_points = latlons.len();
        let ni = grid_def.ni as usize;
        let delta = 3e-5;
        // lat[0], lon[0]
        assert_coord_almost_eq(latlons[0], (-30.4192, 129.906005), delta);
        // lat[0], lon[1]
        assert_coord_almost_eq(latlons[1], (-30.4192, 130.00171406), delta);
        // lat[1], lon[-1]
        assert_coord_almost_eq(latlons[ni], (-30.33658686, 10.71), delta);
        // lat[-2], lon[0]
        assert_coord_almost_eq(
            latlons[num_points - ni - 1],
            (79.9933742, 129.906005),
            delta,
        );
        // lat[-1], lon[-2]
        assert_coord_almost_eq(latlons[num_points - 2], (80.01, 10.61429094), delta);
        // lat[-1], lon[-1]
        assert_coord_almost_eq(latlons[num_points - 1], (80.01, 10.71), delta);

        Ok(())
    }
}
