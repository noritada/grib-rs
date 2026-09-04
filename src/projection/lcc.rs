#[cfg(feature = "gridpoints-proj")]
use super::OsgeoProj;
use super::{
    Ellipsoid, Project,
    helpers::{m, phi2, t},
};

const EPS10: f64 = f64::EPSILON;
const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;
const FORTH_PI: f64 = std::f64::consts::FRAC_PI_4;

/// Parameters for Lambert Conformal Conic projection.
pub struct Params {
    /// Ellipsoid definition
    pub ellipsoid: Ellipsoid,
    /// Latitude of origin (in degree)
    pub lat_0: f64,
    /// Central meridian (in degree)
    pub lon_0: f64,
    /// First standard parallel (in degree)
    pub lat_1: f64,
    /// Second standard parallel (in degree)
    pub lat_2: f64,
}

#[cfg(feature = "gridpoints-proj")]
impl OsgeoProj for Params {
    fn proj_args(&self) -> String {
        let Self {
            ellipsoid: Ellipsoid { a, b, .. },
            lat_0,
            lon_0,
            lat_1,
            lat_2,
        } = self;
        format!(
            "+a={a} +b={b} +proj=lcc +lat_0={lat_0} +lon_0={lon_0} +lat_1={lat_1} +lat_2={lat_2}"
        )
    }
}

pub struct Projection {
    lam0: f64,
    a: f64,
    e: f64,
    e_sq: f64,
    k0: f64,
    n: f64,
    c: f64,
    rho0: f64,
}

impl Projection {
    pub fn new(p: &Params) -> Result<Self, &'static str> {
        let Params {
            ellipsoid: Ellipsoid { a, e, e_sq, .. },
            lat_0,
            lon_0,
            lat_1,
            lat_2,
        } = p;
        let lam0 = lon_0.to_radians();
        let phi0 = lat_0.to_radians();
        let phi1 = lat_1.to_radians();
        let phi2 = lat_2.to_radians();
        let k0 = 1.;

        if (phi1 + phi2).abs() < EPS10 {
            return Err("Invalid value for lat_1 and lat_2: |lat_1 + lat_2| should be > 0");
        }

        let (sinφ1, cosφ1) = phi1.sin_cos();

        if cosφ1.abs() < EPS10 || phi1.abs() >= HALF_PI {
            return Err("Invalid value for lat_1: |lat_1| should be < 90°");
        }
        if phi2.cos().abs() < EPS10 || phi2.abs() >= HALF_PI {
            return Err("Invalid value for lat_2: |lat_2| should be < 90°");
        }

        let is_secant_cone = (phi1 - phi2) >= EPS10; // otherwise, tangent cone
        let is_ellipsoidal = *e_sq != 0.;
        let is_phi0_almost_equal_to_half_pi = (phi0.abs() - HALF_PI).abs() < EPS10;

        let context = if is_ellipsoidal {
            let m1 = m(sinφ1, cosφ1, *e_sq);
            let t1 = t(cosφ1, sinφ1, *e);
            let n = if is_secant_cone {
                n_in_secant_cone_ellipsoidal(phi2, *e, *e_sq, m1, t1)?
            } else {
                sinφ1
            };
            let c = m1 * t1.powf(-n) / n;
            let rho0 = if is_phi0_almost_equal_to_half_pi {
                0.
            } else {
                c * t(phi0.cos(), phi0.sin(), *e).powf(n)
            };
            Projection {
                lam0,
                a: *a,
                e: *e,
                e_sq: *e_sq,
                k0,
                n,
                c,
                rho0,
            }
        } else {
            let n = if is_secant_cone {
                n_in_secant_cone_spherical(phi1, phi2, cosφ1, phi2.cos())
            } else {
                sinφ1
            };
            if n == 0. {
                return Err("Invalid value for lat_1 and lat_2: |lat_1 + lat_2| should be > 0");
            }
            let c = cosφ1 * (FORTH_PI + 0.5 * phi1).tan().powf(n) / n;
            let rho0 = if is_phi0_almost_equal_to_half_pi {
                0.
            } else {
                c * (FORTH_PI + 0.5 * phi0).tan().powf(-n)
            };
            Projection {
                lam0,
                a: *a,
                e: *e,
                e_sq: *e_sq,
                k0,
                n,
                c,
                rho0,
            }
        };
        Ok(context)
    }

    const ERR_TRANSFORMATION_OUTSIDE_DOMAIN: &str =
        "Coordinate transformation outside projection domain";

    fn forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let Self {
            e,
            e_sq,
            k0,
            n,
            c,
            rho0,
            ..
        } = self;

        let rho = if (phi.abs() - HALF_PI).abs() < EPS10 {
            if phi * n <= 0. {
                return Err(Self::ERR_TRANSFORMATION_OUTSIDE_DOMAIN);
            }
            0.
        } else {
            c * if *e_sq != 0. {
                t(phi.cos(), phi.sin(), *e).powf(*n)
            } else {
                (FORTH_PI + 0.5 * phi).tan().powf(-n)
            }
        };
        let lambda = lambda * n;
        let x = k0 * (rho * lambda.sin());
        let y = k0 * (rho0 - rho * lambda.cos());
        Ok((x, y))
    }

    fn inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let Self {
            e,
            e_sq,
            k0,
            n,
            c,
            rho0,
            ..
        } = self;

        let x = x / k0;
        let y = rho0 - y / k0;

        let rho = x.hypot(y);
        let lp = if rho != 0. {
            let (rho, x, y) = if *n < 0. { (-rho, -x, -y) } else { (rho, x, y) };

            let phi = if *e_sq != 0. {
                let phi = phi2((rho / c).powf(1. / n), *e).ok_or(
                    "the inverse of the isometric latitude function could not be solved numerically"
                )?;
                if phi == f64::INFINITY {
                    return Err(Self::ERR_TRANSFORMATION_OUTSIDE_DOMAIN);
                }
                phi
            } else {
                2. * ((c / rho).powf(1. / n)).atan() - HALF_PI
            };
            let lambda = x.atan2(y) / n;
            (lambda, phi)
        } else {
            let phi = if *n > 0. { HALF_PI } else { -HALF_PI };
            (0., phi)
        };

        Ok(lp)
    }
}

impl Project for Projection {
    fn forward(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str> {
        self.forward(xy)
    }

    fn inverse(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str> {
        self.inverse(xy)
    }

    fn a(&self) -> &f64 {
        &self.a
    }

    fn lam0(&self) -> &f64 {
        &self.lam0
    }
}

fn n_in_secant_cone_ellipsoidal(
    phi2: f64,
    e: f64,
    e_sq: f64,
    m1: f64,
    t1: f64,
) -> Result<f64, &'static str> {
    let err_message = "Invalid value for eccentricity";
    let (sinφ2, cosφ2) = phi2.sin_cos();
    let n = (m1 / m(sinφ2, cosφ2, e_sq)).ln();
    if n == 0. {
        return Err(err_message);
    }
    let denom = (t1 / t(cosφ2, sinφ2, e)).ln();
    if denom == 0. {
        return Err(err_message);
    }
    let n = n / denom;
    Ok(n)
}

fn n_in_secant_cone_spherical(phi1: f64, phi2: f64, cosφ1: f64, cosφ2: f64) -> f64 {
    (cosφ1 / cosφ2).ln() / ((FORTH_PI + 0.5 * phi2).tan() / (FORTH_PI + 0.5 * phi1).tan()).ln()
}

#[cfg(all(test, feature = "gridpoints-proj"))]
mod tests {
    use proj::Proj;

    use super::*;

    const FORWARD_TOLERANCE_METERS: f64 = 1e-7;
    const INVERSE_TOLERANCE_RADIANS: f64 = 1e-12;

    #[test]
    fn agrees_with_proj_for_ellipsoidal_secant_cone() {
        assert_agrees_with_proj(Params {
            ellipsoid: Ellipsoid::from_a_and_b(6_378_137., 6_356_752.314_245),
            lat_0: 40.,
            lon_0: 140.,
            lat_1: 60.,
            lat_2: 30.,
        });
    }

    #[test]
    fn agrees_with_proj_for_ellipsoidal_tangent_cone() {
        assert_agrees_with_proj(Params {
            ellipsoid: Ellipsoid::from_a_and_b(6_378_137., 6_356_752.314_245),
            lat_0: 35.,
            lon_0: -100.,
            lat_1: 45.,
            lat_2: 45.,
        });
    }

    #[test]
    fn agrees_with_proj_for_spherical_secant_cone() {
        assert_agrees_with_proj(Params {
            ellipsoid: Ellipsoid::from_a_and_b(6_371_229., 6_371_229.),
            lat_0: -40.,
            lon_0: 20.,
            lat_1: -30.,
            lat_2: -60.,
        });
    }

    fn assert_agrees_with_proj(params: Params) {
        let proj = Proj::new(&params.proj_args()).unwrap();
        let projection = Projection::new(&params).unwrap();
        let coordinates: [(f64, f64); 4] = [(-70., -10.), (0., 0.), (25., 45.), (70., 170.)];

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
