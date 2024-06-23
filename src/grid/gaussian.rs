use super::{GridPointIndexIterator, ScanningMode};
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

// Finds roots (zero points) of the Legendre polynomial using Newton–Raphson
// method.
fn legendre_roots_iterator(n: usize) -> impl Iterator<Item = f32> {
    (0..n).map(move |i| {
        let guess = (i as f32 + 0.5) * std::f32::consts::PI / (n as f32);
        find_root(guess.cos(), |x| {
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
