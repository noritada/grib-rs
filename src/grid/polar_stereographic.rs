#[cfg(feature = "gridpoints-proj")]
use proj::Proj;

use super::{earth::EarthShapeDefinition, GridPointIndexIterator, ScanningMode};
use crate::{
    error::GribError,
    utils::{read_as, GribInt},
    ProjectionCentreFlag,
};

#[derive(Debug, PartialEq, Eq)]
pub struct PolarStereographicGridDefinition {
    pub earth_shape: EarthShapeDefinition,
    pub ni: u32,
    pub nj: u32,
    pub first_point_lat: i32,
    pub first_point_lon: i32,
    pub lad: i32,
    pub lov: i32,
    pub dx: u32,
    pub dy: u32,
    pub projection_centre: ProjectionCentreFlag,
    pub scanning_mode: ScanningMode,
}

impl PolarStereographicGridDefinition {
    /// Returns an iterator over `(i, j)` of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    ///
    /// Examples
    ///
    /// ```
    /// let def = grib::PolarStereographicGridDefinition {
    ///     earth_shape: grib::EarthShapeDefinition {
    ///         shape_of_the_earth: 6,
    ///         scale_factor_of_radius_of_spherical_earth: 0xff,
    ///         scaled_value_of_radius_of_spherical_earth: 0xffffffff,
    ///         scale_factor_of_earth_major_axis: 0xff,
    ///         scaled_value_of_earth_major_axis: 0xffffffff,
    ///         scale_factor_of_earth_minor_axis: 0xff,
    ///         scaled_value_of_earth_minor_axis: 0xffffffff,
    ///     },
    ///     ni: 2,
    ///     nj: 3,
    ///     first_point_lat: 0,
    ///     first_point_lon: 0,
    ///     lad: 0,
    ///     lov: 0,
    ///     dx: 1000,
    ///     dy: 1000,
    ///     projection_centre: grib::ProjectionCentreFlag(0b00000000),
    ///     scanning_mode: grib::ScanningMode(0b01000000),
    /// };
    /// let ij = def.ij();
    /// assert!(ij.is_ok());
    ///
    /// let mut ij = ij.unwrap();
    /// assert_eq!(ij.next(), Some((0, 0)));
    /// assert_eq!(ij.next(), Some((1, 0)));
    /// assert_eq!(ij.next(), Some((0, 1)));
    /// ```
    pub fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        if self.scanning_mode.has_unsupported_flags() {
            let ScanningMode(mode) = self.scanning_mode;
            return Err(GribError::NotSupported(format!("scanning mode {mode}")));
        }

        let iter =
            GridPointIndexIterator::new(self.ni as usize, self.nj as usize, self.scanning_mode);
        Ok(iter)
    }

    /// Returns an iterator over latitudes and longitudes of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    #[cfg(feature = "gridpoints-proj")]
    pub fn latlons(&self) -> Result<std::vec::IntoIter<(f32, f32)>, GribError> {
        let lad = self.lad as f64 * 1e-6;
        let lov = self.lov as f64 * 1e-6;
        let (a, b) = self.earth_shape.radii().ok_or_else(|| {
            GribError::NotSupported(format!(
                "unknown value of Code Table 3.2 (shape of the Earth): {}",
                self.earth_shape.shape_of_the_earth
            ))
        })?;

        if self.projection_centre.has_unsupported_flags() {
            let ProjectionCentreFlag(flag) = self.projection_centre;
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
        let projection = Proj::new(&proj_def).map_err(|e| GribError::Unknown(e.to_string()))?;
        let (first_corner_x, first_corner_y) = projection
            .project(
                (
                    (self.first_point_lon as f64 * 1e-6).to_radians(),
                    (self.first_point_lat as f64 * 1e-6).to_radians(),
                ),
                false,
            )
            .map_err(|e| GribError::Unknown(e.to_string()))?;

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

        let ij = self.ij()?;
        let mut ij = ij
            .map(|(i, j)| {
                (
                    first_corner_x + dx * i as f64,
                    first_corner_y + dy * j as f64,
                )
            })
            .collect::<Vec<_>>();

        let lonlat = projection
            .project_array(&mut ij, true)
            .map_err(|e| GribError::Unknown(e.to_string()))?;
        let latlon = lonlat
            .iter_mut()
            .map(|(lon, lat)| (lat.to_degrees() as f32, lon.to_degrees() as f32))
            .collect::<Vec<_>>();

        Ok(latlon.into_iter())
    }

    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let earth_shape = EarthShapeDefinition::from_buf(buf);
        let ni = read_as!(u32, buf, 16);
        let nj = read_as!(u32, buf, 20);
        let first_point_lat = read_as!(u32, buf, 24).as_grib_int();
        let first_point_lon = read_as!(u32, buf, 28).as_grib_int();
        let lad = read_as!(u32, buf, 33).as_grib_int();
        let lov = read_as!(u32, buf, 37).as_grib_int();
        let dx = read_as!(u32, buf, 41);
        let dy = read_as!(u32, buf, 45);
        let projection_centre = read_as!(u8, buf, 49);
        let scanning_mode = read_as!(u8, buf, 50);
        Self {
            earth_shape,
            ni,
            nj,
            first_point_lat,
            first_point_lon,
            lad,
            lov,
            dx,
            dy,
            projection_centre: ProjectionCentreFlag(projection_centre),
            scanning_mode: ScanningMode(scanning_mode),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use super::*;

    macro_rules! assert_almost_eq {
        ($a1:expr, $a2:expr, $d:expr) => {
            if $a1 - $a2 > $d || $a2 - $a1 > $d {
                panic!();
            }
        };
    }

    fn assert_coord_almost_eq((x1, y1): (f32, f32), (x2, y2): (f32, f32), delta: f32) {
        assert_almost_eq!(x1, x2, delta);
        assert_almost_eq!(y1, y2, delta);
    }

    #[test]
    fn polar_stereographic_grid_definition_from_buf() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();

        let f = std::fs::File::open(
            "testdata/CMC_RDPA_APCP-024-0100cutoff_SFC_0_ps10km_2023121806_000.grib2.xz",
        )?;
        let f = BufReader::new(f);
        let mut f = xz2::bufread::XzDecoder::new(f);
        f.read_to_end(&mut buf)?;

        let actual = PolarStereographicGridDefinition::from_buf(&buf[0x33..]);
        let expected = PolarStereographicGridDefinition {
            earth_shape: EarthShapeDefinition {
                shape_of_the_earth: 6,
                scale_factor_of_radius_of_spherical_earth: 0xff,
                scaled_value_of_radius_of_spherical_earth: 0xffffffff,
                scale_factor_of_earth_major_axis: 0xff,
                scaled_value_of_earth_major_axis: 0xffffffff,
                scale_factor_of_earth_minor_axis: 0xff,
                scaled_value_of_earth_minor_axis: 0xffffffff,
            },
            ni: 935,
            nj: 824,
            first_point_lat: 18145030,
            first_point_lon: 217107456,
            lad: 60000000,
            lov: 249000000,
            dx: 10000000,
            dy: 10000000,
            projection_centre: ProjectionCentreFlag(0b00000000),
            scanning_mode: ScanningMode(0b01000000),
        };
        assert_eq!(actual, expected);

        Ok(())
    }

    #[cfg(feature = "gridpoints-proj")]
    #[test]
    fn polar_stereographic_grid_latlon_computation() -> Result<(), Box<dyn std::error::Error>> {
        let grid_def = PolarStereographicGridDefinition {
            earth_shape: EarthShapeDefinition {
                shape_of_the_earth: 6,
                scale_factor_of_radius_of_spherical_earth: 0xff,
                scaled_value_of_radius_of_spherical_earth: 0xffffffff,
                scale_factor_of_earth_major_axis: 0xff,
                scaled_value_of_earth_major_axis: 0xffffffff,
                scale_factor_of_earth_minor_axis: 0xff,
                scaled_value_of_earth_minor_axis: 0xffffffff,
            },
            ni: 935,
            nj: 824,
            first_point_lat: 18145030,
            first_point_lon: 217107456,
            lad: 60000000,
            lov: 249000000,
            dx: 10000000,
            dy: 10000000,
            projection_centre: ProjectionCentreFlag(0b00000000),
            scanning_mode: ScanningMode(0b01000000),
        };
        let latlons = grid_def.latlons()?.collect::<Vec<_>>();

        // Following lat/lon values are taken from the calculation results using pygrib.
        let delta = 1e-10;
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
