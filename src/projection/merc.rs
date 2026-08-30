use super::{
    Ellipsoid, MercParams,
    helpers::{m, sinhpsi2tanphi},
};

const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;

struct MercDefinition {
    lam0: f64,
    phi_ts: f64,
    e: f64,
    e_sq: f64,
}

impl From<&MercParams> for MercDefinition {
    fn from(value: &MercParams) -> Self {
        let MercParams {
            ellipsoid: Ellipsoid { e, e_sq, .. },
            lat_ts,
            lon_0,
        } = value;

        let lam0 = lon_0.to_radians();
        let phi_ts = lat_ts.to_radians().abs();

        Self {
            lam0,
            phi_ts,
            e: *e,
            e_sq: *e_sq,
        }
    }
}

pub struct Projection {
    params: MercDefinition,
    k0: f64,
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
        let context = Projection { params: p, k0 };
        Ok(context)
    }

    pub fn project(&self, xy: &(f64, f64), inverse: bool) -> Result<(f64, f64), &'static str> {
        match (self.params.e_sq == 0.0, inverse) {
            (true, true) => self.spheroidal_inverse(xy),
            (true, false) => self.spheroidal_forward(xy),
            (false, true) => self.ellipsoidal_inverse(xy),
            (false, false) => self.ellipsoidal_forward(xy),
        }
    }

    fn ellipsoidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let x = self.k0 * lambda;
        let (sinφ, cosφ) = phi.sin_cos();
        let y = self.k0 * ((sinφ / cosφ).asinh() - self.params.e * (self.params.e * sinφ).atanh());
        Ok((x, y))
    }

    fn spheroidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let x = self.k0 * lambda;
        let y = self.k0 * phi.tan().asinh();
        Ok((x, y))
    }

    fn ellipsoidal_inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let phi = sinhpsi2tanphi((y / self.k0).sinh(), self.params.e).ok_or(
            "the inverse of the isometric latitude function could not be solved numerically",
        )?;
        let lambda = x / self.k0;
        Ok((lambda, phi))
    }

    fn spheroidal_inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let phi = (y / self.k0).sinh().atan();
        let lambda = x / self.k0;
        Ok((lambda, phi))
    }
}
