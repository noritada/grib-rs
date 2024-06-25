use super::{
    helpers::{evenly_spaced_degrees, RegularGridIterator},
    GridPointIndexIterator, ScanningMode,
};
use crate::{
    error::GribError,
    utils::{read_as, GribInt},
};

#[derive(Debug, PartialEq, Eq)]
pub struct GaussianGridDefinition {
    pub ni: u32,
    pub nj: u32,
    pub first_point_lat: i32,
    pub first_point_lon: i32,
    pub last_point_lat: i32,
    pub last_point_lon: i32,
    pub i_direction_inc: u32,
    pub n: u32,
    pub scanning_mode: ScanningMode,
}

impl GaussianGridDefinition {
    /// Returns the shape of the grid, i.e. a tuple of the number of grids in
    /// the i and j directions.
    pub fn grid_shape(&self) -> (usize, usize) {
        (self.ni as usize, self.nj as usize)
    }

    /// Returns the grid type.
    pub fn short_name(&self) -> &'static str {
        "regular_gg"
    }

    /// Returns an iterator over `(i, j)` of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn ij(&self) -> Result<GridPointIndexIterator, GribError> {
        if self.scanning_mode.has_unsupported_flags() {
            let ScanningMode(mode) = self.scanning_mode;
            return Err(GribError::NotSupported(format!("scanning mode {mode}")));
        }

        let iter =
            GridPointIndexIterator::new(self.ni as usize, self.nj as usize, self.scanning_mode);
        Ok(iter)
    }

    /// Returns an iterator over latitudes and longitudes of grid points.
    ///
    /// Note that this is a low-level API and it is not checked that the number
    /// of iterator iterations is consistent with the number of grid points
    /// defined in the data.
    pub fn latlons(&self) -> Result<RegularGridIterator, GribError> {
        if !self.is_consistent() {
            return Err(GribError::InvalidValueError("Latitude and longitude for first/last grid points are not consistent with scanning mode".to_owned()));
        }

        let ij = self.ij()?;
        let mut lat = compute_gaussian_latitudes(self.nj as usize)
            .map_err(|e| GribError::Unknown(e.to_owned()))?;
        if self.scanning_mode.scans_positively_for_j() {
            lat.reverse()
        };
        let lon = evenly_spaced_degrees(
            self.first_point_lon as f32,
            self.last_point_lon as f32,
            (self.ni - 1) as usize,
        );

        let iter = RegularGridIterator::new(lat, lon, ij);
        Ok(iter)
    }

    pub(crate) fn is_consistent(&self) -> bool {
        let lat_diff = self.last_point_lat - self.first_point_lat;
        let lon_diff = self.last_point_lon - self.first_point_lon;
        !(((lat_diff > 0) ^ self.scanning_mode.scans_positively_for_j())
            || ((lon_diff > 0) ^ self.scanning_mode.scans_positively_for_i()))
    }

    pub(crate) fn from_buf(buf: &[u8]) -> Self {
        let ni = read_as!(u32, buf, 0);
        let nj = read_as!(u32, buf, 4);
        let first_point_lat = read_as!(u32, buf, 16).as_grib_int();
        let first_point_lon = read_as!(u32, buf, 20).as_grib_int();
        let last_point_lat = read_as!(u32, buf, 25).as_grib_int();
        let last_point_lon = read_as!(u32, buf, 29).as_grib_int();
        let i_direction_inc = read_as!(u32, buf, 33);
        let n = read_as!(u32, buf, 37);
        let scanning_mode = read_as!(u8, buf, 41);
        Self {
            ni,
            nj,
            first_point_lat,
            first_point_lon,
            last_point_lat,
            last_point_lon,
            i_direction_inc,
            n,
            scanning_mode: ScanningMode(scanning_mode),
        }
    }
}

fn compute_gaussian_latitudes(div: usize) -> Result<Vec<f32>, &'static str> {
    let lat: Vec<_> = legendre_roots_iterator(div)
        .map(|i| i.asin().to_degrees())
        .collect();
    Ok(lat)
}

// Finds roots (zero points) of the Legendre polynomial using Newton–Raphson
// method.
fn legendre_roots_iterator(n: usize) -> impl Iterator<Item = f32> {
    let coeff = 1.0_f32 - 1.0 / (8 * n * n) as f32 + 1.0 / (8 * n * n * n) as f32;
    (0..n).map(move |i| {
        // Francesco G. Tricomi, Sugli zeri dei polinomi sferici ed ultrasferici, Annali di Matematica Pura ed Applicata, 31 (1950), pp. 93–97.
        // F.G. Lether, P.R. Wenston, Minimax approximations to the zeros of Pn(x) and Gauss-Legendre quadrature, Journal of Computational and Applied Mathematics, Volume 59, Issue 2, 1995, Pages 245-252, ISSN 0377-0427, https://doi.org/10.1016/0377-0427(94)00030-5.
        let guess = coeff * ((4 * i + 3) as f32 * std::f32::consts::PI / (4 * n + 2) as f32).cos();
        find_root(guess, |x| {
            let (p_prev, p) = legendre_polynomial(n, x);
            let fpx = legendre_polynomial_derivative(n, x, p_prev, p);
            p / fpx
        })
    })
}

// `n` is assumed to be greater than or equal to 2.
fn legendre_polynomial(n: usize, x: f32) -> (f32, f32) {
    let mut p0 = 1.0;
    let mut p1 = x;
    for k in 2..=n {
        let pk = ((2 * k - 1) as f32 * x * p1 - (k - 1) as f32 * p0) / k as f32;
        p0 = p1;
        p1 = pk;
    }
    (p0, p1)
}

fn legendre_polynomial_derivative(n: usize, x: f32, p_prev: f32, p: f32) -> f32 {
    (n as f32 * (p_prev - x * p)) / (1.0 - x * x)
}

// Finds a root (zero point) of the given function using Newton–Raphson method.
fn find_root<F>(initial_guess: f32, f: F) -> f32
where
    F: Fn(f32) -> f32,
{
    let mut x = initial_guess;
    loop {
        let dx = f(x);
        x -= dx;
        if dx.abs() < f32::EPSILON {
            break;
        }
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::helpers::test_helpers::assert_almost_eq;

    macro_rules! test_legendre_roots_iterator_with_analytical_solutions {
        ($((
            $name:ident,
            $n:expr,
            $expected:expr,
        ),)*) => ($(
            #[test]
            fn $name() {
                let actual = legendre_roots_iterator($n).collect::<Vec<_>>();
                let expected = $expected;
                assert_eq!(actual, expected);
            }
        )*);
    }

    test_legendre_roots_iterator_with_analytical_solutions! {
        (
            legendre_roots_iterator_for_n_being_2_compared_with_analytical_solutions,
            2,
            vec![1.0 / 3.0_f32.sqrt(), -1.0 / 3.0_f32.sqrt()],
        ),
        (
            legendre_roots_iterator_for_n_being_5_compared_with_analytical_solutions,
            5,
            vec![
                (5.0_f32 + 2.0 * (10.0_f32 / 7.0).sqrt()).sqrt() / 3.0,
                (5.0_f32 - 2.0 * (10.0_f32 / 7.0).sqrt()).sqrt() / 3.0,
                0.0,
                - (5.0_f32 - 2.0 * (10.0_f32 / 7.0).sqrt()).sqrt() / 3.0,
                - (5.0_f32 + 2.0 * (10.0_f32 / 7.0).sqrt()).sqrt() / 3.0,
            ],
        ),
    }

    // Values are copied and pasted from ["Features for ERA-40 grids"](https://web.archive.org/web/20160925045844/http://rda.ucar.edu/datasets/common/ecmwf/ERA40/docs/std-transformations/dss_code_glwp.html).
    #[test]
    fn gaussian_latitudes_computation_compared_with_numerical_solutions() {
        let n = 160;
        let result = compute_gaussian_latitudes(n);
        assert!(result.is_ok());

        let actual = result.unwrap().into_iter().take(n / 2);
        let expected = "
                    +   89.1416,  88.0294,  86.9108,  85.7906,  84.6699,  83.5489,
                    +   82.4278,  81.3066,  80.1853,  79.0640,  77.9426,  76.8212,
                    +   75.6998,  74.5784,  73.4570,  72.3356,  71.2141,  70.0927,
                    +   68.9712,  67.8498,  66.7283,  65.6069,  64.4854,  63.3639,
                    +   62.2425,  61.1210,  59.9995,  58.8780,  57.7566,  56.6351,
                    +   55.5136,  54.3921,  53.2707,  52.1492,  51.0277,  49.9062,
                    +   48.7847,  47.6632,  46.5418,  45.4203,  44.2988,  43.1773,
                    +   42.0558,  40.9343,  39.8129,  38.6914,  37.5699,  36.4484,
                    +   35.3269,  34.2054,  33.0839,  31.9624,  30.8410,  29.7195,
                    +   28.5980,  27.4765,  26.3550,  25.2335,  24.1120,  22.9905,
                    +   21.8690,  20.7476,  19.6261,  18.5046,  17.3831,  16.2616,
                    +   15.1401,  14.0186,  12.8971,  11.7756,  10.6542,   9.5327,
                    +    8.4112,   7.2897,   6.1682,   5.0467,   3.9252,   2.8037,
                    +    1.6822,   0.5607 /
                    ";
        let expected = expected
            .split(&['+', ' ', ',', '\n', '/'])
            .filter_map(|s| s.parse::<f32>().ok());

        let delta = 1.0e-4;
        for (actual_val, expected_val) in actual.zip(expected) {
            assert_almost_eq!(actual_val, expected_val, delta);
        }
    }

    #[test]
    fn finding_root() {
        let actual = find_root(1.0, |x| {
            let fx = x * x - 2.0;
            let fpx = x * 2.0;
            fx / fpx
        });
        let expected = 1.41421356;
        assert_almost_eq!(actual, expected, 1.0e-8)
    }
}
