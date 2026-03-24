#[cfg(feature = "gridpoints-proj")]
use crate::error::GribError;
use crate::{
    GridPointIndex, LatLons,
    def::grib2::template::{Template3_20, param_set},
};

impl crate::GridShortName for Template3_20 {
    fn short_name(&self) -> &'static str {
        "polar_stereographic"
    }
}

impl GridPointIndex for Template3_20 {
    fn grid_shape(&self) -> (usize, usize) {
        (self.ni as usize, self.nj as usize)
    }

    fn scanning_mode(&self) -> &param_set::ScanningMode {
        &self.scanning_mode
    }
}

#[cfg(feature = "gridpoints-proj")]
#[cfg_attr(docsrs, doc(cfg(feature = "gridpoints-proj")))]
impl LatLons for Template3_20 {
    type Iter<'a>
        = std::vec::IntoIter<(f32, f32)>
    where
        Self: 'a;

    fn latlons_unchecked<'a>(&'a self) -> Result<Self::Iter<'a>, GribError> {
        let lad = self.lad as f64 * 1e-6;
        let lov = self.lov as f64 * 1e-6;
        let (a, b) = self.earth_shape.radii().ok_or_else(|| {
            GribError::NotSupported(format!(
                "unknown value of Code Table 3.2 (shape of the Earth): {}",
                self.earth_shape.shape
            ))
        })?;

        if self.projection_centre.has_unsupported_flags() {
            let param_set::ProjectionCentreFlag(flag) = self.projection_centre;
            return Err(GribError::NotSupported(format!("projection centre {flag}")));
        }
        let lat_origin = if self
            .projection_centre
            .contains_north_pole_on_projection_plane()
        {
            90.
        } else {
            -90.
        };

        let proj_def =
            format!("+a={a} +b={b} +proj=stere +lat_ts={lad} +lat_0={lat_origin} +lon_0={lov}");

        let dx = self.dx as f64 * 1e-3;
        let dy = self.dy as f64 * 1e-3;
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

        super::helpers::latlons_from_projection_definition_and_first_point(
            &proj_def,
            (
                self.first_point_lat as f64 * 1e-6,
                self.first_point_lon as f64 * 1e-6,
            ),
            (dx, dy),
            self.ij()?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "gridpoints-proj")]
    #[test]
    fn polar_stereographic_grid_latlon_computation() -> Result<(), Box<dyn std::error::Error>> {
        use crate::grid::helpers::test_helpers::assert_coord_almost_eq;
        // grid point definition extracted from
        // testdata/CMC_RDPA_APCP-024-0100cutoff_SFC_0_ps10km_2023121806_000.grib2.xz
        let grid_def = Template3_20 {
            earth_shape: param_set::EarthShape {
                shape: 6,
                spherical_earth_radius_scale_factor: 0xff,
                spherical_earth_radius_scaled_value: 0xffffffff,
                major_axis_scale_factor: 0xff,
                major_axis_scaled_value: 0xffffffff,
                minor_axis_scale_factor: 0xff,
                minor_axis_scaled_value: 0xffffffff,
            },
            ni: 935,
            nj: 824,
            first_point_lat: 18145030,
            first_point_lon: 217107456,
            resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00001000),
            lad: 60000000,
            lov: 249000000,
            dx: 10000000,
            dy: 10000000,
            projection_centre: param_set::ProjectionCentreFlag(0b00000000),
            scanning_mode: param_set::ScanningMode(0b01000000),
        };
        let latlons = grid_def.latlons()?.collect::<Vec<_>>();

        // Following lat/lon values are taken from the calculation results using pygrib.
        let delta = 1e-4;
        assert_coord_almost_eq(latlons[0], (18.14503, -142.892544), delta);
        assert_coord_almost_eq(latlons[1], (18.17840149, -142.83604096), delta);
        assert_coord_almost_eq(
            latlons[latlons.len() - 2],
            (45.4865147, -10.15230394),
            delta,
        );
        assert_coord_almost_eq(
            latlons[latlons.len() - 1],
            (45.40545211, -10.17442147),
            delta,
        );

        Ok(())
    }
}
