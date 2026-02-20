use super::GridPointIndexIterator;
use crate::{
    LatLonGridDefinition,
    error::GribError,
    grid::helpers::RegularGridIterator,
    helpers::{GribInt, read_as},
};

#[derive(Debug, PartialEq)]
pub struct RotatedLatLonGridDefinition {
    pub rotated: LatLonGridDefinition,
    pub rotation: Rotation,
}

impl RotatedLatLonGridDefinition {
    /// Returns the shape of the grid, i.e. a tuple of the number of grids in
    /// the i and j directions.
    pub fn grid_shape(&self) -> (usize, usize) {
        self.rotated.grid_shape()
    }

    /// Returns the grid type.
    pub fn short_name(&self) -> &'static str {
        "rotated_ll"
    }

    /// Returns an iterator over `(i, j)` of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        self.rotated.ij()
    }

    /// Returns an iterator over latitudes and longitudes of grid points in
    /// degrees.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn latlons(&self) -> Result<Unrotate<RegularGridIterator>, GribError> {
        let iter = Unrotate::new(self.rotated.latlons()?, &self.rotation);
        Ok(iter)
    }

    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let lat_lon = LatLonGridDefinition::from_buf(buf);
        let south_pole_lat = read_as!(u32, buf, 42).as_grib_int();
        let south_pole_lon = read_as!(u32, buf, 46).as_grib_int();
        let rot_angle = read_as!(f32, buf, 50);
        let rotation = Rotation {
            south_pole_lat,
            south_pole_lon,
            rot_angle,
        };
        Self {
            rotated: lat_lon,
            rotation,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rotation {
    pub south_pole_lat: i32,
    pub south_pole_lon: i32,
    pub rot_angle: f32,
}

#[derive(Clone)]
pub struct Unrotate<I> {
    latlons: I,
    sinφp: f32,
    cosφp: f32,
    λp: f32,
    gamma: f32,
}

impl<I> Unrotate<I> {
    fn new(latlons: I, rot: &Rotation) -> Self {
        let φp = (rot.south_pole_lat as f32 * 1e-6).to_radians();
        let λp = (rot.south_pole_lon as f32 * 1e-6).to_radians();
        let gamma = rot.rot_angle.to_radians();

        // south pole to north pole
        let φp = -φp;
        let λp = λp + std::f32::consts::PI;

        let (sinφp, cosφp) = φp.sin_cos();
        Self {
            latlons,
            sinφp,
            cosφp,
            λp,
            gamma,
        }
    }
}

impl<I> Iterator for Unrotate<I>
where
    I: Iterator<Item = (f32, f32)>,
{
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        let (lat, lon) = self.latlons.next()?;
        let λr = lon.to_radians();
        let φr = lat.to_radians();

        let λr = λr - self.gamma;

        let (sinφr, cosφr) = φr.sin_cos();
        let (sinλr, cosλr) = λr.sin_cos();

        let sinφ = self.sinφp * sinφr + self.cosφp * cosφr * cosλr;
        let φ = sinφ.asin();

        let y = cosφr * sinλr;
        let x = self.cosφp * sinφr - self.sinφp * cosφr * cosλr;
        let λ = self.λp - y.atan2(x);

        let latlon = (φ.to_degrees(), λ.to_degrees());
        Some(latlon)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.latlons.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;

    #[test]
    fn rotated_latlon_grid_definition_from_buf() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = Vec::new();

        let f = std::fs::File::open(
            "testdata/20260219T00Z_MSC_HRDPS_CAPE_Sfc_RLatLon0.0225_PT000H.grib2",
        )?;
        let mut f = std::io::BufReader::new(f);
        f.read_to_end(&mut buf)?;

        let actual = RotatedLatLonGridDefinition::from_buf(&buf[0x43..]);
        let expected = RotatedLatLonGridDefinition {
            rotated: LatLonGridDefinition {
                ni: 2540,
                nj: 1290,
                first_point_lat: -12302501,
                first_point_lon: 345178780,
                last_point_lat: 16700001,
                last_point_lon: 42306283,
                scanning_mode: crate::ScanningMode(0b01000000),
            },
            rotation: Rotation {
                south_pole_lat: -36088520,
                south_pole_lon: 245305142,
                rot_angle: 0.,
            },
        };
        assert_eq!(actual, expected);

        Ok(())
    }

    macro_rules! test_rotation{
        ($(($name:ident, $rot:expr, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let rot = $rot;
                let latlons = vec![$input];
                let mut iter = Unrotate::new(latlons.into_iter(), &rot);
                let actual = iter.next().unwrap();
                let expected = $expected;
                dbg!(actual);
                dbg!(expected);
                assert!((actual.0 - expected.0).abs() < 0.00001);
                assert!((actual.1 - expected.1).abs() < 0.001);
            }
        )*);
    }

    test_rotation! {
        (
            no_rotation,
            Rotation {
                south_pole_lat: -90000000,
                south_pole_lon: 0,
                rot_angle: 0.,
            },
            (-12.302501_f32, 345.178780_f32),
            (-12.302501_f32, 345.178780_f32)
        ),
        (
            rotation_for_first_point,
            Rotation {
                south_pole_lat: -36088520,
                south_pole_lon: 245305142,
                rot_angle: 0.,
            },
            (-12.302501_f32, 345.178780_f32),
            // taken from results from pygrib
            (39.626032, -133.62952 + 720.)
        ),
    }
}
