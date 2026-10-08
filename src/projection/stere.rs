#[cfg(feature = "gridpoints-proj")]
use super::OsgeoProj;
use super::{Ellipsoid, Project, helpers::t};

const HALF_PI: f64 = std::f64::consts::FRAC_PI_2;
const FORTH_PI: f64 = std::f64::consts::FRAC_PI_4;
const EPS10: f64 = 1e-10;
const TOL: f64 = 1e-8;

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

/// Stereographic projection.
pub struct Projection {
    lam0: f64,
    a: f64,
    e: f64,
    e_sq: f64,
    phi0: f64,
    akm1: f64,
    mode: ProjectionMode,
}

impl Projection {
    pub fn new(p: &Params) -> Result<Self, &'static str> {
        let Params {
            ellipsoid: Ellipsoid { a, e, e_sq, .. },
            lat_ts,
            lat_0,
            lon_0,
            k_0,
        } = p;
        let phi0 = lat_0.to_radians();
        let lam0 = lon_0.to_radians();
        let phi_ts = lat_ts.to_radians().abs();

        let absφ0 = phi0.abs();
        let mode = if (absφ0 - HALF_PI).abs() < EPS10 {
            ProjectionMode::Pole {
                is_north_pole: phi0.is_sign_positive(),
            }
        } else {
            ProjectionMode::NonPole {
                is_equator: absφ0 <= EPS10,
                x: ConformalLatitude::default(),
            }
        };

        let (mode, akm1) = if *e_sq == 0. {
            match mode {
                ProjectionMode::NonPole { is_equator, .. } => {
                    let mode = if is_equator {
                        mode
                    } else {
                        ProjectionMode::NonPole {
                            is_equator,
                            x: ConformalLatitude::from(phi0),
                        }
                    };
                    let akm1 = 2. * k_0;
                    (mode, akm1)
                }
                ProjectionMode::Pole { .. } => {
                    let akm1 = if (phi_ts - HALF_PI).abs() < EPS10 {
                        2. * k_0
                    } else {
                        phi_ts.cos() / (FORTH_PI - 0.5 * phi_ts).tan()
                    };
                    (mode, akm1)
                }
            }
        } else {
            match mode {
                ProjectionMode::NonPole { is_equator, .. } => {
                    let (sinφ0, cosφ0) = phi0.sin_cos();
                    // conformal latitude (latitude on the conformal sphere)
                    let x = 2. * psi(phi0, sinφ0, *e).atan() - HALF_PI;
                    let t = sinφ0 * e;
                    let akm1 = 2. * k_0 * cosφ0 / (1. - t * t).sqrt();
                    let mode = ProjectionMode::NonPole {
                        is_equator,
                        x: ConformalLatitude::from(x),
                    };
                    (mode, akm1)
                }
                ProjectionMode::Pole { .. } => {
                    let akm1 = if (phi_ts - HALF_PI).abs() < EPS10 {
                        2. * k_0 / ((1. + e).powf(1. + e) * (1. - e).powf(1. - e)).sqrt()
                    } else {
                        let (sinφts, cosφts) = phi_ts.sin_cos();
                        let akm1 = cosφts / t(cosφts, sinφts, *e);
                        let t = sinφts * e;
                        akm1 / (1. - t * t).sqrt()
                    };
                    (mode, akm1)
                }
            }
        };

        let context = Self {
            lam0,
            a: *a,
            e: *e,
            e_sq: *e_sq,
            phi0,
            akm1,
            mode,
        };
        Ok(context)
    }

    fn ellipsoidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let (sinλ, cosλ) = lambda.sin_cos();
        let (sinφ, cosφ) = phi.sin_cos();

        let (x, y) = match &self.mode {
            ProjectionMode::NonPole {
                is_equator,
                x: ConformalLatitude { sinφ0, cosφ0 },
            } => {
                let x = 2. * psi(*phi, sinφ, self.e).atan() - HALF_PI;
                let (sinx, cosx) = x.sin_cos();

                if *is_equator {
                    if 1. + cosx * cosλ == 0.0 {
                        (0., f64::INFINITY)
                    } else {
                        let a = self.akm1 / (1. + cosx * cosλ);
                        let x = a * cosx;
                        let y = a * sinx;
                        (x, y)
                    }
                } else {
                    let denom = cosφ0 * (1. + sinφ0 * sinx + cosφ0 * cosx * cosλ);
                    if denom == 0. {
                        return Err(Self::ERR_TRANSFORMATION_OUTSIDE_DOMAIN);
                    }
                    let a = self.akm1 / denom;
                    let x = a * cosx;
                    let y = a * (cosφ0 * sinx - sinφ0 * cosx * cosλ);
                    (x, y)
                }
            }
            ProjectionMode::Pole { is_north_pole } => {
                let (phi, cosλ, sinφ) = if *is_north_pole {
                    (*phi, cosλ, sinφ)
                } else {
                    (-phi, -cosλ, -sinφ)
                };
                let x = if (phi - HALF_PI).abs() < 1e-15 {
                    0.
                } else {
                    self.akm1 * t(cosφ, sinφ, self.e)
                };
                let y = -x * cosλ;
                (x, y)
            }
        };

        let x = x * sinλ;
        Ok((x, y))
    }

    const ERR_TRANSFORMATION_OUTSIDE_DOMAIN: &str =
        "Coordinate transformation outside projection domain";

    fn spheroidal_forward(&self, (lambda, phi): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let (sinλ, cosλ) = lambda.sin_cos();
        let (sinφ, cosφ) = phi.sin_cos();

        let (x, y) = match &self.mode {
            ProjectionMode::NonPole {
                is_equator,
                x: ConformalLatitude { sinφ0, cosφ0 },
            } => {
                let y = if *is_equator {
                    1. + cosφ * cosλ
                } else {
                    1. + sinφ0 * sinφ + cosφ0 * cosφ * cosλ
                };

                if y <= EPS10 {
                    return Err(Self::ERR_TRANSFORMATION_OUTSIDE_DOMAIN);
                }

                let c = self.akm1 / y;
                let x = c * cosφ * sinλ;
                let y = c * if *is_equator {
                    sinφ
                } else {
                    cosφ0 * sinφ - sinφ0 * cosφ * cosλ
                };
                (x, y)
            }
            ProjectionMode::Pole { is_north_pole } => {
                let (cosλ, phi) = if *is_north_pole {
                    (-cosλ, -phi)
                } else {
                    (cosλ, *phi)
                };

                if (phi - HALF_PI).abs() < TOL {
                    return Err(Self::ERR_TRANSFORMATION_OUTSIDE_DOMAIN);
                }

                let c = self.akm1 * (FORTH_PI + 0.5 * phi).tan();
                let x = c * sinλ;
                let y = c * cosλ;
                (x, y)
            }
        };

        Ok((x, y))
    }

    fn ellipsoidal_inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        const MAX_ITER: usize = 8;
        const CONV: f64 = 1e-10;
        let rho = x.hypot(*y);

        let (x, y, tp, mut phi_l, half_pi, half_e) = match &self.mode {
            ProjectionMode::NonPole {
                x: ConformalLatitude { sinφ0, cosφ0 },
                ..
            } => {
                let tp = 2. * (rho * cosφ0).atan2(self.akm1);
                let (sinφ, cosφ) = tp.sin_cos();
                let phi_l = if rho == 0. {
                    (cosφ * sinφ0).asin()
                } else {
                    (cosφ * sinφ0 + (y * sinφ * cosφ0 / rho)).asin()
                };

                let tp = (0.5 * (HALF_PI + phi_l)).tan();
                let x = x * sinφ;
                let y = rho * cosφ0 * cosφ - y * sinφ0 * sinφ;
                let half_pi = HALF_PI;
                let half_e = 0.5 * self.e;
                (x, y, tp, phi_l, half_pi, half_e)
            }
            ProjectionMode::Pole { is_north_pole } => {
                let y = if *is_north_pole { -y } else { *y };
                let tp = -rho / self.akm1;
                let phi_l = HALF_PI - 2. * tp.atan();
                let half_pi = -HALF_PI;
                let half_e = -0.5 * self.e;
                (*x, y, tp, phi_l, half_pi, half_e)
            }
        };

        let mut count = MAX_ITER;
        while count > 0 {
            let sinφ = self.e * phi_l.sin();
            let phi = 2. * (tp * ((1. + sinφ) / (1. - sinφ)).powf(half_e)).atan() - half_pi;
            if (phi_l - phi).abs() < CONV {
                let phi = if let ProjectionMode::Pole {
                    is_north_pole: false,
                } = self.mode
                {
                    -phi
                } else {
                    phi
                };
                let lambda = if x == 0. && y == 0. { 0. } else { x.atan2(y) };
                return Ok((lambda, phi));
            }

            phi_l = phi;
            count -= 1;
        }

        Err(Self::ERR_TRANSFORMATION_OUTSIDE_DOMAIN)
    }

    fn spheroidal_inverse(&self, (x, y): &(f64, f64)) -> Result<(f64, f64), &'static str> {
        let rho = x.hypot(*y);
        let c = 2. * (rho / self.akm1).atan();
        let (sinc, cosc) = c.sin_cos();

        let (lambda, phi) = match &self.mode {
            ProjectionMode::NonPole {
                is_equator: true, ..
            } => {
                let phi = if rho.abs() <= EPS10 {
                    0.
                } else {
                    (y * sinc / rho).asin()
                };
                let lambda = if cosc == 0. && *x == 0. {
                    0.
                } else {
                    (x * sinc).atan2(cosc * rho)
                };
                (lambda, phi)
            }
            ProjectionMode::NonPole {
                is_equator: false,
                x: ConformalLatitude { sinφ0, cosφ0 },
            } => {
                let phi = if rho.abs() <= EPS10 {
                    self.phi0
                } else {
                    (cosc * sinφ0 + y * sinc * cosφ0 / rho).asin()
                };
                let c = cosc - sinφ0 * phi.sin();
                let lambda = if c == 0. && *x == 0. {
                    0.
                } else {
                    (x * sinc * cosφ0).atan2(c * rho)
                };
                (lambda, phi)
            }
            ProjectionMode::Pole { is_north_pole } => {
                let (y, cosc) = if *is_north_pole {
                    (-y, cosc)
                } else {
                    (*y, -cosc)
                };
                let phi = if rho.abs() <= EPS10 {
                    self.phi0
                } else {
                    cosc.asin()
                };
                let lambda = if *x == 0. && y == 0. { 0. } else { x.atan2(y) };
                (lambda, phi)
            }
        };

        Ok((lambda, phi))
    }
}

impl Project for Projection {
    fn forward(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str> {
        if self.e_sq == 0.0 {
            self.spheroidal_forward(xy)
        } else {
            self.ellipsoidal_forward(xy)
        }
    }

    fn inverse(&self, xy: &(f64, f64)) -> Result<(f64, f64), &'static str> {
        if self.e_sq == 0.0 {
            self.spheroidal_inverse(xy)
        } else {
            self.ellipsoidal_inverse(xy)
        }
    }

    fn a(&self) -> &f64 {
        &self.a
    }

    fn lam0(&self) -> &f64 {
        &self.lam0
    }
}

enum ProjectionMode {
    NonPole {
        is_equator: bool,
        x: ConformalLatitude,
    },
    Pole {
        is_north_pole: bool,
    },
}

#[derive(Default)]
struct ConformalLatitude {
    sinφ0: f64,
    cosφ0: f64,
}

impl From<f64> for ConformalLatitude {
    fn from(value: f64) -> Self {
        let (sinφ0, cosφ0) = value.sin_cos();
        Self { sinφ0, cosφ0 }
    }
}

// isometric latitude
fn psi(phit: f64, sinφ: f64, e: f64) -> f64 {
    let sinφ = sinφ * e;
    (0.5 * (HALF_PI + phit)).tan() * ((1. - sinφ) / (1. + sinφ)).powf(0.5 * e)
}

#[cfg(all(test, feature = "gridpoints-proj"))]
mod tests {
    use proj::Proj;

    use super::*;

    const FORWARD_TOLERANCE_METERS: f64 = 1e-7;
    const INVERSE_TOLERANCE_RADIANS: f64 = 1e-12;

    macro_rules! projection_comparison_tests {
        ($(($name:ident, $params:expr, $coordinates:expr),)*) => ($(
            #[test]
            fn $name() -> Result<(), Box<dyn std::error::Error>> {
                assert_agrees_with_proj($params, $coordinates)
            }
        )*);
    }

    projection_comparison_tests! {
        (
            agrees_with_proj_for_ellipsoidal_north_pole,
            Params {
                ellipsoid: Ellipsoid::from_a_and_b(6_378_137., 6_356_752.314_245),
                lat_ts: 60.,
                lat_0: 90.,
                lon_0: -111.,
                k_0: 1.,
            },
            [(20., -140.), (45., -100.), (70., 10.), (89., 170.)]
        ),
        (
            agrees_with_proj_for_ellipsoidal_south_pole,
            Params {
                ellipsoid: Ellipsoid::from_a_and_b(6_378_137., 6_356_752.314_245),
                lat_ts: -71.,
                lat_0: -90.,
                lon_0: 20.,
                k_0: 1.,
            },
            [(-20., -160.), (-45., -30.), (-70., 80.), (-89., 170.)]
        ),
        (
            agrees_with_proj_for_spherical_north_pole,
            Params {
                ellipsoid: Ellipsoid::from_a_and_b(6_371_229., 6_371_229.),
                lat_ts: 60.,
                lat_0: 90.,
                lon_0: 140.,
                k_0: 1.,
            },
            [(20., -170.), (45., -40.), (70., 100.), (89., 179.)]
        ),
        (
            agrees_with_proj_for_spherical_equatorial,
            Params {
                ellipsoid: Ellipsoid::from_a_and_b(6_371_229., 6_371_229.),
                lat_ts: 0.,
                lat_0: 0.,
                lon_0: 15.,
                k_0: 0.9996,
            },
            [(-70., -20.), (-20., 0.), (25., 45.), (70., 100.)]
        ),
        (
            agrees_with_proj_for_ellipsoidal_oblique,
            Params {
                ellipsoid: Ellipsoid::from_a_and_b(6_378_137., 6_356_752.314_245),
                lat_ts: 40.,
                lat_0: 40.,
                lon_0: -100.,
                k_0: 0.9996,
            },
            [(-20., -140.), (0., -100.), (45., -70.), (75., 20.)]
        ),
    }

    fn assert_agrees_with_proj(
        params: Params,
        coordinates: [(f64, f64); 4],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let proj = Proj::new(&params.proj_args())?;
        let projection = Projection::new(&params)?;

        for (lat, lon) in coordinates {
            let lonlat = (lon.to_radians(), lat.to_radians());
            let expected_xy = proj.project(lonlat, false)?;
            let actual_xy = projection.project(&lonlat, false)?;
            assert_coordinates_close(actual_xy, expected_xy, FORWARD_TOLERANCE_METERS);

            let expected_lonlat = proj.project(expected_xy, true)?;
            let actual_lonlat = projection.project(&expected_xy, true)?;
            assert_coordinates_close(actual_lonlat, expected_lonlat, INVERSE_TOLERANCE_RADIANS);
        }

        Ok(())
    }

    fn assert_coordinates_close(actual: (f64, f64), expected: (f64, f64), tolerance: f64) {
        assert!(
            (actual.0 - expected.0).abs() <= tolerance
                && (actual.1 - expected.1).abs() <= tolerance,
            "actual {actual:?} differs from expected {expected:?} by more than {tolerance}"
        );
    }
}
