use std::f64::consts::PI;

use super::{
    Ellipsoid, LccParams,
    helpers::{m, phi2, t},
};

// FIXME: use constant values defined in proj
const EPS10: f64 = f64::EPSILON;
const HALF_PI: f64 = PI / 2.;
const FORTH_PI: f64 = PI / 4.;

struct LccDefinition {
    lam0: f64,
    phi0: f64,
    phi1: f64,
    phi2: f64,
    e: f64,
    e_sq: f64,
    k0: f64,
}

impl From<&LccParams> for LccDefinition {
    fn from(value: &LccParams) -> Self {
        let LccParams {
            ellipsoid: Ellipsoid { e, e_sq, .. },
            lat_0,
            lon_0,
            lat_1,
            lat_2,
        } = value;

        let lam0 = lon_0.to_radians();
        let phi0 = lat_0.to_radians();
        let phi1 = lat_1.to_radians();
        let phi2 = lat_2.to_radians();

        Self {
            lam0,
            phi0,
            phi1,
            phi2,
            e: *e,
            e_sq: *e_sq,
            k0: 1.,
        }
    }
}

pub struct Projection {
    params: LccDefinition,
    n: f64,
    c: f64,
    rho0: f64,
}

impl Projection {
    pub fn new(p: &LccParams) -> Result<Self, &'static str> {
        let p = LccDefinition::from(p);
        if (p.phi1 + p.phi2).abs() < EPS10 {
            return Err("Invalid value for lat_1 and lat_2: |lat_1 + lat_2| should be > 0");
        }

        let sin_phi1 = p.phi1.sin();
        let cos_phi1 = p.phi1.cos();

        if cos_phi1.abs() < EPS10 || p.phi1.abs() >= HALF_PI {
            return Err("Invalid value for lat_1: |lat_1| should be < 90°");
        }
        if p.phi2.cos().abs() < EPS10 || p.phi2.abs() >= HALF_PI {
            return Err("Invalid value for lat_2: |lat_2| should be < 90°");
        }

        let is_secant_cone = (p.phi1 - p.phi2) >= EPS10; // otherwise, tangent cone
        let is_ellipsoidal = p.e_sq != 0.;
        let is_phi0_almost_equal_to_half_pi = (p.phi0.abs() - HALF_PI).abs() < EPS10;

        let context = if is_ellipsoidal {
            let m1 = m(sin_phi1, cos_phi1, p.e_sq);
            let t1 = t(cos_phi1, sin_phi1, p.e);
            let n = if is_secant_cone {
                n_in_secant_cone_ellipsoidal(&p, &m1, &t1)?
            } else {
                sin_phi1
            };
            let c = m1 * t1.powf(-n) / n;
            let rho0 = if is_phi0_almost_equal_to_half_pi {
                0.
            } else {
                c * t(p.phi0.cos(), p.phi0.sin(), p.e).powf(n)
            };
            Projection {
                params: p,
                n,
                c,
                rho0,
            }
        } else {
            let n = if is_secant_cone {
                n_in_secant_cone_spherical(&p, cos_phi1, p.phi2.cos())
            } else {
                sin_phi1
            };
            if n == 0. {
                return Err("Invalid value for lat_1 and lat_2: |lat_1 + lat_2| should be > 0");
            }
            let c = cos_phi1 * (FORTH_PI + 0.5 * p.phi1).tan().powf(n) / n;
            let rho0 = if is_phi0_almost_equal_to_half_pi {
                0.
            } else {
                c * (FORTH_PI + 0.5 * p.phi0).tan().powf(-n)
            };
            Projection {
                params: p,
                n,
                c,
                rho0,
            }
        };
        Ok(context)
    }

    const ERR_TRANSFORMATION_OUTSIDE_DOMAIN: &str =
        "Coordinate transformation outside projection domain";

    pub fn project(&self, xy: &(f64, f64), inverse: bool) -> Result<(f64, f64), &'static str> {
        if inverse {
            self.inverse(xy)
        } else {
            self.forward(xy)
        }
    }

    fn forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let Self {
            params: LccDefinition { e, e_sq, k0, .. },
            n,
            c,
            rho0,
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
            params: LccDefinition { e, e_sq, k0, .. },
            n,
            c,
            rho0,
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

fn n_in_secant_cone_ellipsoidal(
    p: &LccDefinition,
    m1: &f64,
    t1: &f64,
) -> Result<f64, &'static str> {
    let err_message = "Invalid value for eccentricity";
    let sin_phi2 = p.phi2.sin();
    let cos_phi2 = p.phi2.cos();
    let n = (m1 / m(sin_phi2, cos_phi2, p.e_sq)).ln();
    if n == 0. {
        return Err(err_message);
    }
    let denom = (t1 / t(cos_phi2, sin_phi2, p.e)).ln();
    if denom == 0. {
        return Err(err_message);
    }
    let n = n / denom;
    Ok(n)
}

fn n_in_secant_cone_spherical(p: &LccDefinition, cos_phi1: f64, cos_phi2: f64) -> f64 {
    (cos_phi1 / cos_phi2).ln()
        / ((FORTH_PI + 0.5 * p.phi2).tan() / (FORTH_PI + 0.5 * p.phi1).tan()).ln()
}
