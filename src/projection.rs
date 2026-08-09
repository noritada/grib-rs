pub enum ProjectionParams {
    Lcc(LccParams),
    Stere(StereParams),
}

impl ProjectionParams {
    pub(crate) fn osgeo_proj_args(&self) -> String {
        match self {
            Self::Lcc(p) => p.proj_args(),
            Self::Stere(p) => p.proj_args(),
        }
    }
}

pub struct LccParams {
    pub a: f64,
    pub b: f64,
    pub lat_0: f64,
    pub lon_0: f64,
    pub lat_1: f64,
    pub lat_2: f64,
}

impl LccParams {
    fn proj_args(&self) -> String {
        let Self {
            a,
            b,
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

pub struct StereParams {
    pub a: f64,
    pub b: f64,
    pub lat_ts: f64,
    pub lat_0: f64,
    pub lon_0: f64,
}

impl StereParams {
    fn proj_args(&self) -> String {
        let Self {
            a,
            b,
            lat_ts,
            lat_0,
            lon_0,
        } = self;
        format!("+a={a} +b={b} +proj=stere +lat_ts={lat_ts} +lat_0={lat_0} +lon_0={lon_0}")
    }
}
