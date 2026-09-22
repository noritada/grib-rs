use super::Ellipsoid;
#[cfg(feature = "gridpoints-proj")]
use super::OsgeoProj;

/// Parameters for stereographic projection.
#[derive(Debug, PartialEq, Clone)]
pub struct Params {
    /// Ellipsoid definition.
    pub ellipsoid: Ellipsoid,
    /// Latitude where scale is not distorted (in degrees).
    pub lat_ts: f64,
    /// Latitude of origin (in degrees).
    pub lat_0: f64,
    /// Central meridian (in degrees).
    pub lon_0: f64,
    /// Scale factor.
    pub k_0: f64,
}

#[cfg(feature = "gridpoints-proj")]
impl OsgeoProj for Params {
    fn proj_args(&self) -> String {
        let Self {
            ellipsoid: Ellipsoid { a, b, .. },
            lat_ts,
            lat_0,
            lon_0,
            k_0,
        } = self;
        format!(
            "+a={a} +b={b} +proj=stere +lat_ts={lat_ts} +lat_0={lat_0} +lon_0={lon_0} +k_0={k_0}"
        )
    }
}
