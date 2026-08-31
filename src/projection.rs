#![allow(unused)] // FIXME

pub enum Projection {
    Lcc(lcc::Projection),
    Merc(merc::Projection),
}

impl Projection {
    pub fn new(params: &ProjectionParams) -> Result<Self, &'static str> {
        let inner = match params {
            ProjectionParams::Lcc(p) => Self::Lcc(lcc::Projection::new(p)?),
            ProjectionParams::Merc(p) => Self::Merc(merc::Projection::new(p)?),
            ProjectionParams::Stere(_) => todo!(),
        };
        Ok(inner)
    }

    pub fn project(&self, xy: &(f64, f64), inverse: bool) -> Result<(f64, f64), &'static str> {
        match self {
            Self::Lcc(inner) => inner.project(xy, inverse),
            Self::Merc(inner) => inner.project(xy, inverse),
        }
    }
}

pub trait Project {
    fn forward(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str>;
    fn inverse(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str>;
    fn a(&self) -> &f64;
    fn lam0(&self) -> &f64;

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

/// Projection parameters.
pub enum ProjectionParams {
    /// Lambert Conformal Conic projection
    Lcc(LccParams),
    /// Mercator projection
    Merc(MercParams),
    /// Stereographic projection
    Stere(StereParams),
}

impl ProjectionParams {
    pub(crate) fn osgeo_proj_args(&self) -> String {
        match self {
            Self::Lcc(p) => p.proj_args(),
            Self::Merc(p) => p.proj_args(),
            Self::Stere(p) => p.proj_args(),
        }
    }
}

/// Parameters for Lambert Conformal Conic projection.
pub struct LccParams {
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

impl LccParams {
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

/// Parameters for Mercator projection.
pub struct MercParams {
    /// Ellipsoid definition
    pub ellipsoid: Ellipsoid,
    /// Latitude of true scale (in degree)
    pub lat_ts: f64,
    /// Central meridian (in degree)
    pub lon_0: f64,
}

impl MercParams {
    fn proj_args(&self) -> String {
        let Self {
            ellipsoid: Ellipsoid { a, b, .. },
            lat_ts,
            lon_0,
        } = self;
        format!("+a={a} +b={b} +proj=merc +lat_ts={lat_ts} +lon_0={lon_0}")
    }
}

/// Parameters for Stereographic projection.
pub struct StereParams {
    /// Ellipsoid definition
    pub ellipsoid: Ellipsoid,
    /// Latitude where scale is not distorted (in degree)
    pub lat_ts: f64,
    /// Latitude of origin (in degree)
    pub lat_0: f64,
    /// Central meridian (in degree)
    pub lon_0: f64,
}

impl StereParams {
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

pub struct Ellipsoid {
    /// Semimajor radius of the ellipsoid axis (in meters)
    pub a: f64,
    /// Semiminor radius of the ellipsoid axis (in meters)
    pub b: f64,
    /// Eccentricity
    pub e: f64,
    /// Eccentricity squared
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
