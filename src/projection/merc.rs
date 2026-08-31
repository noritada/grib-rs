use super::{
    Ellipsoid, MercParams, Project,
    helpers::{m, sinhpsi2tanphi},
};

const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;

struct MercDefinition {
    lam0: f64,
    phi_ts: f64,
    a: f64,
    e: f64,
    e_sq: f64,
}

impl From<&MercParams> for MercDefinition {
    fn from(value: &MercParams) -> Self {
        let MercParams {
            ellipsoid: Ellipsoid { a, e, e_sq, .. },
            lat_ts,
            lon_0,
        } = value;

        let lam0 = lon_0.to_radians();
        let phi_ts = lat_ts.to_radians().abs();

        Self {
            lam0,
            phi_ts,
            a: *a,
            e: *e,
            e_sq: *e_sq,
        }
    }
}

pub struct Projection {
    params: MercDefinition,
    ak0: f64,
}

impl Projection {
    pub fn new(p: &MercParams) -> Result<Self, &'static str> {
        let p = MercDefinition::from(p);
        if p.phi_ts >= HALF_PI {
            return Err("Invalid value for lat_ts: |lat_ts| should be <= 90°");
        }

        let k0 = if p.e_sq == 0.0 {
            // sphere
            p.phi_ts.cos()
        } else {
            // ellipsoid
            let (sinφts, cosφts) = p.phi_ts.sin_cos();
            m(sinφts, cosφts, p.e_sq)
        };
        let ak0 = p.a * k0;
        let context = Projection { params: p, ak0 };
        Ok(context)
    }

    fn ellipsoidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let &x = lambda;
        let (sinφ, cosφ) = phi.sin_cos();
        let y = (sinφ / cosφ).asinh() - self.params.e * (self.params.e * sinφ).atanh();
        Ok((x, y))
    }

    fn spheroidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let &x = lambda;
        let y = phi.tan().asinh();
        Ok((x, y))
    }

    fn ellipsoidal_inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let phi = sinhpsi2tanphi(y.sinh(), self.params.e)
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
        if self.params.e_sq == 0.0 {
            self.spheroidal_forward(xy)
        } else {
            self.ellipsoidal_forward(xy)
        }
    }

    fn inverse(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str> {
        if self.params.e_sq == 0.0 {
            self.spheroidal_inverse(xy)
        } else {
            self.ellipsoidal_inverse(xy)
        }
    }

    fn a(&self) -> &f64 {
        &self.ak0
    }

    fn lam0(&self) -> &f64 {
        &self.params.lam0
    }
}
