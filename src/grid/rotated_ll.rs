use super::GridPointIndexIterator;
use crate::{
    GridPointIndex, LatLons,
    def::grib2::template::{Template3_1, param_set::Rotation},
    error::GribError,
    grid::{AngleUnit, helpers::RegularGridIterator},
};

impl crate::GridShortName for Template3_1 {
    fn short_name(&self) -> &'static str {
        "rotated_ll"
    }
}

impl GridPointIndex for Template3_1 {
    fn grid_shape(&self) -> (usize, usize) {
        self.rotated.grid_shape()
    }

    fn scanning_mode(&self) -> &crate::def::grib2::template::param_set::ScanningMode {
        self.rotated.scanning_mode()
    }

    fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        self.rotated.ij()
    }
}

impl LatLons for Template3_1 {
    type Iter<'a>
        = Unrotate<RegularGridIterator>
    where
        Self: 'a;

    fn latlons_unchecked<'a>(&'a self) -> Result<Self::Iter<'a>, GribError> {
        let iter = Unrotate::new(
            self.rotated.latlons_unchecked()?,
            &self.rotation,
            self.angle_unit() as f32,
        );
        Ok(iter)
    }
}

impl AngleUnit for Template3_1 {
    fn angle_unit(&self) -> f64 {
        self.rotated.grid.angle_unit()
    }
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
    fn new(latlons: I, rot: &Rotation, angle_units: f32) -> Self {
        let φp = (rot.south_pole_lat as f32 * angle_units).to_radians();
        let λp = (rot.south_pole_lon as f32 * angle_units).to_radians();
        let gamma = (rot.rot_angle * angle_units).to_radians();

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

    use super::*;

    macro_rules! test_rotation{
        ($(($name:ident, $rot:expr, $input:expr, $expected:expr),)*) => ($(
            #[test]
            fn $name() {
                let rot = $rot;
                let latlons = vec![$input];
                let mut iter = Unrotate::new(latlons.into_iter(), &rot, 1e-6);
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
            // grid point definition extracted from
            // testdata/20260219T00Z_MSC_HRDPS_CAPE_Sfc_RLatLon0.0225_PT000H.grib2
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
