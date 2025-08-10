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
