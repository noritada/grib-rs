use std::f64::consts::PI;

// FIXME: use constant values defined in proj
const EPS10: f64 = f64::EPSILON;
const HALF_PI: f64 = PI / 2.;
const FORTH_PI: f64 = PI / 4.;

pub struct Params {
    phi0: f64,
    phi1: f64,
    phi2: f64,
    e: f64,
    e_sq: f64,
    k0: f64,
}

pub struct Projection {
    params: Params,
    n: f64,
    c: f64,
    rho0: f64,
}

impl Projection {
    pub fn new(p: Params) -> Result<Self, &'static str> {
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
            params: Params { e, e_sq, k0, .. },
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
            params: Params { e, e_sq, k0, .. },
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

fn n_in_secant_cone_ellipsoidal(p: &Params, m1: &f64, t1: &f64) -> Result<f64, &'static str> {
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

fn n_in_secant_cone_spherical(p: &Params, cos_phi1: f64, cos_phi2: f64) -> f64 {
    (cos_phi1 / cos_phi2).ln()
        / ((FORTH_PI + 0.5 * p.phi2).tan() / (FORTH_PI + 0.5 * p.phi1).tan()).ln()
}

fn m(sin_phi: f64, cos_phi: f64, e_sq: f64) -> f64 {
    cos_phi / (1. - e_sq * sin_phi * sin_phi).sqrt()
}

// See formula deformation for pj_tsfn().
fn t(cos_phi: f64, sin_phi: f64, e: f64) -> f64 {
    (e * (e * sin_phi).atanh()).exp()
        * if sin_phi > 0. {
            cos_phi / (1. + sin_phi)
        } else {
            (1. - sin_phi) / cos_phi
        }
}

// See formula deformation for pj_phi2().
fn phi2(ts0: f64, e: f64) -> Option<f64> {
    let phi2 = sinhpsi2tanphi((1. / ts0 - ts0) / 2., e)?.atan();
    Some(phi2)
}

// See formula deformation for pj_sinhpsi2tanphi().
fn sinhpsi2tanphi(taup: f64, e: f64) -> Option<f64> {
    const MAX_ITER: usize = 5;
    let root_eps: f64 = f64::EPSILON.sqrt();
    let tol: f64 = root_eps / 10.; // the criterion for Newton's method
    let tmax: f64 = 2. / root_eps; // threshold for large arg limit exact
    let e2m: f64 = 1. - e * e;
    let stol: f64 = tol * 1.0_f64.max(taup.abs());

    // The initial guess.  70 corresponds to chi = 89.18 deg
    let mut tau = if taup.abs() > 70. {
        taup * (e * e.atanh()).exp()
    } else {
        taup / e2m
    };

    // handles +/-inf and nan and e = 1
    if !(tau.abs() < tmax) {
        return Some(tau);
    }

    let mut count = MAX_ITER;
    while count > 0 {
        let tau1 = (1. + tau * tau).sqrt();
        let sig = (e * (e * tau / tau1).atanh()).sinh();
        let taupa = (1. + sig * sig).sqrt() * tau - sig * tau1;
        let dtau =
            (taup - taupa) * (1. + e2m * (tau * tau)) / (e2m * tau1 * (1. + taupa * taupa).sqrt());

        tau += dtau;

        // backwards test to allow nans to succeed.
        if !(dtau.abs() >= stol) {
            return Some(tau);
        }

        count -= 1;
    }
    None
}
