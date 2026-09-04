use super::{
    Ellipsoid, OsgeoProj, Project,
    helpers::{m, sinhpsi2tanphi},
};

const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;

/// Parameters for Mercator projection.
pub struct Params {
    /// Ellipsoid definition
    pub ellipsoid: Ellipsoid,
    /// Latitude of true scale (in degree)
    pub lat_ts: f64,
    /// Central meridian (in degree)
    pub lon_0: f64,
}

impl OsgeoProj for Params {
    fn proj_args(&self) -> String {
        let Self {
            ellipsoid: Ellipsoid { a, b, .. },
            lat_ts,
            lon_0,
        } = self;
        format!("+a={a} +b={b} +proj=merc +lat_ts={lat_ts} +lon_0={lon_0}")
    }
}

pub struct Projection {
    lam0: f64,
    e: f64,
    e_sq: f64,
    ak0: f64,
}

impl Projection {
    pub fn new(p: &Params) -> Result<Self, &'static str> {
        let Params {
            ellipsoid: Ellipsoid { a, e, e_sq, .. },
            lat_ts,
            lon_0,
        } = p;
        let lam0 = lon_0.to_radians();
        let phi_ts = lat_ts.to_radians().abs();
        if phi_ts >= HALF_PI {
            return Err("Invalid value for lat_ts: |lat_ts| should be <= 90°");
        }

        let k0 = if *e_sq == 0.0 {
            // sphere
            phi_ts.cos()
        } else {
            // ellipsoid
            let (sinφts, cosφts) = phi_ts.sin_cos();
            m(sinφts, cosφts, *e_sq)
        };
        let ak0 = a * k0;
        let context = Projection {
            lam0,
            e: *e,
            e_sq: *e_sq,
            ak0,
        };
        Ok(context)
    }

    fn ellipsoidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let &x = lambda;
        let (sinφ, cosφ) = phi.sin_cos();
        let y = (sinφ / cosφ).asinh() - self.e * (self.e * sinφ).atanh();
        Ok((x, y))
    }

    fn spheroidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let &x = lambda;
        let y = phi.tan().asinh();
        Ok((x, y))
    }

    fn ellipsoidal_inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let phi = sinhpsi2tanphi(y.sinh(), self.e)
            .ok_or(
                "the inverse of the isometric latitude function could not be solved numerically",
            )?
            .atan();
        let &lambda = x;
        Ok((lambda, phi))
    }

    fn spheroidal_inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let phi = y.sinh().atan();
        let &lambda = x;
        Ok((lambda, phi))
    }
}

impl Project for Projection {
    fn forward(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str> {
        if self.e_sq == 0.0 {
            self.spheroidal_forward(xy)
        } else {
            self.ellipsoidal_forward(xy)
        }
    }

    fn inverse(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str> {
        if self.e_sq == 0.0 {
            self.spheroidal_inverse(xy)
        } else {
            self.ellipsoidal_inverse(xy)
        }
    }

    fn a(&self) -> &f64 {
        &self.ak0
    }

    fn lam0(&self) -> &f64 {
        &self.lam0
    }
}

#[cfg(all(test, feature = "gridpoints-proj"))]
mod tests {
    use proj::Proj;

    use super::*;

    const FORWARD_TOLERANCE_METERS: f64 = 1e-8;
    const INVERSE_TOLERANCE_RADIANS: f64 = 1e-12;

    #[test]
    fn agrees_with_proj_for_ellipsoid() {
        assert_agrees_with_proj(Params {
            ellipsoid: Ellipsoid::from_a_and_b(6_378_137., 6_356_752.314_245),
            lat_ts: 20.,
            lon_0: 140.,
        });
    }

    #[test]
    fn agrees_with_proj_for_sphere() {
        assert_agrees_with_proj(Params {
            ellipsoid: Ellipsoid::from_a_and_b(6_371_229., 6_371_229.),
            lat_ts: -15.,
            lon_0: -30.,
        });
    }

    fn assert_agrees_with_proj(params: Params) {
        let proj = Proj::new(&params.proj_args()).unwrap();
        let projection = Projection::new(&params).unwrap();
        let coordinates: [(f64, f64); 4] = [(-10., -70.), (0., 0.), (25., 45.), (80., 170.)];

        for (lat, lon) in coordinates {
            let lonlat = (lon.to_radians(), lat.to_radians());
            let expected_xy = proj.project(lonlat, false).unwrap();
            let actual_xy = projection.project(&lonlat, false).unwrap();
            assert_coordinates_close(actual_xy, expected_xy, FORWARD_TOLERANCE_METERS);

            let expected_lonlat = proj.project(expected_xy, true).unwrap();
            let actual_lonlat = projection.project(&expected_xy, true).unwrap();
            assert_coordinates_close(actual_lonlat, expected_lonlat, INVERSE_TOLERANCE_RADIANS);
        }
    }

    fn assert_coordinates_close(actual: (f64, f64), expected: (f64, f64), tolerance: f64) {
        assert!(
            (actual.0 - expected.0).abs() <= tolerance
                && (actual.1 - expected.1).abs() <= tolerance,
            "actual {actual:?} differs from expected {expected:?} by more than {tolerance}"
        );
    }
}
