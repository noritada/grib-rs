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

        let angle_units = self.angle_unit();
        let (a, b) = self.earth_shape.radii().ok_or_else(|| {
            GribError::NotSupported(format!(
                "unknown value of Code Table 3.2 (shape of the Earth): {}",
                self.earth_shape.shape
            ))
        })?;
        let params = projection::MercParams {
            ellipsoid: projection::Ellipsoid::from_a_and_b(a, b),
            lat_ts: self.lad as f64 * angle_units,
            lon_0: 0.,
        };

        let dx = self.di as f64 * 1e-3;
        let dy = self.dj as f64 * 1e-3;
        let dx = if !self.scanning_mode.scans_positively_for_i() && dx > 0. {
            -dx
        } else {
            dx
        };
        let dy = if !self.scanning_mode.scans_positively_for_j() && dy > 0. {
            -dy
        } else {
            dy
        };

        let first_point = (
            self.first_point_lat as f64 * angle_units,
            self.first_point_lon as f64 * angle_units,
        );

        #[cfg(feature = "gridpoints-proj")]
        {
            super::helpers::latlons_from_projection_definition_and_first_point(
                &params.proj_args(),
                first_point,
                (dx, dy),
                self.ij()?,
            )
        }

        #[cfg(not(feature = "gridpoints-proj"))]
        {
            let projection =
                projection::Merc::new(&params).map_err(|e| GribError::Unknown(e.to_owned()))?;
            let (first_point_lat, first_point_lon) = first_point;
            let (first_corner_x, first_corner_y) = projection
                .project(
                    &(first_point_lon.to_radians(), first_point_lat.to_radians()),
                    false,
                )
                .map_err(|e| GribError::Unknown(e.to_owned()))?;

            let latlons = self
                .ij()?
                .map(|(i, j)| {
                    projection.project(
                        &(
                            first_corner_x + dx * i as f64,
                            first_corner_y + dy * j as f64,
                        ),
                        true,
                    )
                })
                .map(|result| {
                    result
                        .map(|(lon, lat)| (lat.to_degrees() as f32, lon.to_degrees() as f32))
                        .map_err(|e| GribError::Unknown(e.to_owned()))
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
