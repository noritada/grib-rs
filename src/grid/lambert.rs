#[cfg(feature = "gridpoints-proj")]
use crate::LatLons;
use crate::{
    GridPointIndex,
    def::grib2::template::{Template3_30, param_set},
    error::GribError,
};

impl crate::GridShortName for Template3_30 {
    fn short_name(&self) -> &'static str {
        "lambert"
    }
}

impl GridPointIndex for Template3_30 {
    fn grid_shape(&self) -> (usize, usize) {
        (self.ni as usize, self.nj as usize)
    }

    fn scanning_mode(&self) -> &param_set::ScanningMode {
        &self.scanning_mode
    }
}

#[cfg(feature = "gridpoints-proj")]
#[cfg_attr(docsrs, doc(cfg(feature = "gridpoints-proj")))]
impl LatLons for Template3_30 {
    type Iter<'a> = std::vec::IntoIter<(f32, f32)>;

    fn latlons<'a>(&'a self) -> Result<Self::Iter<'a>, GribError> {
        let lad = self.lad as f64 * 1e-6;
        let lov = self.lov as f64 * 1e-6;
        let latin1 = self.latin1 as f64 * 1e-6;
        let latin2 = self.latin2 as f64 * 1e-6;
        let (a, b) = self.earth_shape.radii().ok_or_else(|| {
            GribError::NotSupported(format!(
                "unknown value of Code Table 3.2 (shape of the Earth): {}",
                self.earth_shape.shape
            ))
        })?;
        let proj_def = format!(
            "+a={a} +b={b} +proj=lcc +lat_0={lad} +lon_0={lov} +lat_1={latin1} +lat_2={latin2}"
        );

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
    fn lambert_grid_latlon_computation() -> Result<(), Box<dyn std::error::Error>> {
        use crate::grid::helpers::test_helpers::assert_coord_almost_eq;
        // grid point definition extracted from testdata/ds.critfireo.bin.xz
        let grid_def = Template3_30 {
            earth_shape: param_set::EarthShape {
                shape: 1,
                spherical_earth_radius_scale_factor: 0,
                spherical_earth_radius_scaled_value: 6371200,
                major_axis_scale_factor: 0,
                major_axis_scaled_value: 0,
                minor_axis_scale_factor: 0,
                minor_axis_scaled_value: 0,
            },
            ni: 2145,
            nj: 1377,
            first_point_lat: 20190000,
            first_point_lon: 238449996,
            resolution_and_component_flags: param_set::ResolutionAndComponentFlags(0b00000000),
            lad: 25000000,
            lov: 265000000,
            dx: 2539703,
            dy: 2539703,
            projection_centre: param_set::ProjectionCentreFlag(0b00000000),
            scanning_mode: param_set::ScanningMode(0b01010000),
            latin1: 25000000,
            latin2: 25000000,
            south_pole_lat: -90000000,
            south_pole_lon: 0,
        };
        let latlons = grid_def.latlons()?.collect::<Vec<_>>();

        // Following lat/lon values are taken from the calculation results using pygrib.
        let delta = 1e-10;
        assert_coord_almost_eq(latlons[0], (20.19, -121.550004), delta);
        assert_coord_almost_eq(latlons[1], (20.19442682, -121.52621665), delta);
        assert_coord_almost_eq(
            latlons[latlons.len() - 2],
            (50.10756403, -60.91298217),
            delta,
        );
        assert_coord_almost_eq(
            latlons[latlons.len() - 1],
            (50.1024611, -60.88202274),
            delta,
        );

        Ok(())
    }
}
