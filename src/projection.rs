//! Map projection functionality.

pub use lcc::{Params as LccParams, Projection as Lcc};
pub use merc::{Params as MercParams, Projection as Merc};

/// Map projection functionality.
pub trait Project {
    fn forward(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str>;
    fn inverse(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str>;
    fn a(&self) -> &f64;
    fn lam0(&self) -> &f64;

    /// Performs a projection.
    ///
    /// For forward transformation (`inverse = false`), `xy` is treated as
    /// `(lambda, phi)`, where `lambda` represents longitude and `phi`
    /// represents latitude, both expressed in degrees. The return value
    /// should be considered to be `(x, y)`, and both `x` and `y` are values in
    /// meters.
    ///
    /// For inverse transformation (`inverse = true`), `xy` is treated as `(x,
    /// y)` and both `x` and `y` are values in meters.
    /// The return value should be considered to be `(lambda, phi)`, where
    /// `lambda` represents longitude and `phi` represents latitude, both
    /// expressed in degrees.
    fn project(&self, xy: &(f64, f64), inverse: bool) -> Result<(f64, f64), &'static str> {
        if inverse {
            let &(x, y) = xy;
            let x = x / self.a();
            let y = y / self.a();
            let (lambda, phi) = self.inverse(&(x, y))?;
            let lambda = helpers::normalize_longitude(lambda + self.lam0());
            Ok((lambda, phi))
        } else {
            let &(lambda, phi) = xy;
            let lambda = helpers::normalize_longitude(lambda - self.lam0());
            let (x, y) = self.forward(&(lambda, phi))?;
            let x = x * self.a();
            let y = y * self.a();
            Ok((x, y))
        }
    }
}

#[cfg(feature = "gridpoints-proj")]
pub(crate) trait OsgeoProj {
    fn proj_args(&self) -> String;
}

/// Parameters for Stereographic projection.
#[derive(Debug, PartialEq, Clone)]
pub struct StereParams {
    /// Ellipsoid definition.
    pub ellipsoid: Ellipsoid,
    /// Latitude where scale is not distorted (in degrees).
    pub lat_ts: f64,
    /// Latitude of origin (in degrees).
    pub lat_0: f64,
    /// Central meridian (in degrees).
    pub lon_0: f64,
}

#[cfg(feature = "gridpoints-proj")]
impl OsgeoProj for StereParams {
    fn proj_args(&self) -> String {
        let Self {
            ellipsoid: Ellipsoid { a, b, .. },
            lat_ts,
            lat_0,
            lon_0,
        } = self;
        format!("+a={a} +b={b} +proj=stere +lat_ts={lat_ts} +lat_0={lat_0} +lon_0={lon_0}")
    }
}

/// Ellipsoid definition.
#[derive(Debug, PartialEq, Clone)]
pub struct Ellipsoid {
    /// Semimajor radius of the ellipsoid axis (in meters).
    pub a: f64,
    /// Semiminor radius of the ellipsoid axis (in meters).
    pub b: f64,
    /// Eccentricity.
    pub e: f64,
    /// Eccentricity squared.
    pub e_sq: f64,
}

impl Ellipsoid {
    pub fn from_a_and_b(a: f64, b: f64) -> Self {
        let f = (a - b) / a;
        let e_sq = 2. * f - f * f;
        let e = e_sq.sqrt();
        Self { a, b, e, e_sq }
    }
}

mod helpers;
mod lcc;
mod merc;
