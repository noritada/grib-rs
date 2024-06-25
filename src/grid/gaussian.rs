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

    #[test]
    fn latlon_computation_for_real_world_gaussian_grid_compared_with_results_from_eccodes(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::Read;

        let mut buf = Vec::new();

        let f = std::fs::File::open("testdata/gdas.t00z.sfluxgrbf000.grib2.0.xz")?;
        let f = std::io::BufReader::new(f);
        let mut f = xz2::bufread::XzDecoder::new(f);
        f.read_to_end(&mut buf)?;

        let f = std::io::Cursor::new(buf);
        let grib2 = crate::from_reader(f)?;

        let ((_, _), first_submessage) = grib2
            .submessages()
            .next()
            .ok_or_else(|| Box::<dyn std::error::Error>::from("first submessage not found"))?;
        let grid_shape = first_submessage.grid_shape()?;
        assert_eq!(grid_shape, (3072, 1536));

        let delta = 1.0e-6;

        // Results from the following command line using ecCodes:
        //
        // ```
        // xzcat testdata/gdas.t00z.sfluxgrbf000.grib2.0.xz \
        //     | grib_get_data -m foo -L "%11.6f%11.6f" - \
        //     | grep -v '^Latitude' | awk '{print $1;}' | uniq | head -160
        // ```
        let first_160_lats_expected = "
                89.910325 89.794157 89.677304 89.560296 89.443229 89.326134 89.209022 89.091901
                88.974774 88.857642 88.740506 88.623369 88.506229 88.389088 88.271946 88.154803
                88.037660 87.920515 87.803370 87.686225 87.569079 87.451933 87.334787 87.217640
                87.100493 86.983346 86.866199 86.749052 86.631904 86.514757 86.397609 86.280461
                86.163313 86.046165 85.929017 85.811869 85.694721 85.577572 85.460424 85.343275
                85.226127 85.108979 84.991830 84.874681 84.757533 84.640384 84.523236 84.406087
                84.288938 84.171789 84.054641 83.937492 83.820343 83.703194 83.586045 83.468896
                83.351747 83.234599 83.117450 83.000301 82.883152 82.766003 82.648854 82.531705
                82.414556 82.297407 82.180258 82.063109 81.945960 81.828811 81.711662 81.594512
                81.477363 81.360214 81.243065 81.125916 81.008767 80.891618 80.774469 80.657320
                80.540171 80.423021 80.305872 80.188723 80.071574 79.954425 79.837276 79.720126
                79.602977 79.485828 79.368679 79.251530 79.134381 79.017231 78.900082 78.782933
                78.665784 78.548635 78.431485 78.314336 78.197187 78.080038 77.962888 77.845739
                77.728590 77.611441 77.494292 77.377142 77.259993 77.142844 77.025695 76.908545
                76.791396 76.674247 76.557098 76.439948 76.322799 76.205650 76.088501 75.971351
                75.854202 75.737053 75.619904 75.502754 75.385605 75.268456 75.151306 75.034157
                74.917008 74.799859 74.682709 74.565560 74.448411 74.331262 74.214112 74.096963
                73.979814 73.862664 73.745515 73.628366 73.511217 73.394067 73.276918 73.159769
                73.042619 72.925470 72.808321 72.691172 72.574022 72.456873 72.339724 72.222574
                72.105425 71.988276 71.871126 71.753977 71.636828 71.519679 71.402529 71.285380
            ";
        let first_160_lats_expected = first_160_lats_expected
            .split_whitespace()
            .filter_map(|s| s.parse::<f32>().ok());

        let first_160_lats = first_submessage
            .latlons()?
            .map(|(lat, _lon)| lat)
            .step_by(3072)
            .take(160);
        for (actual, expected) in first_160_lats.zip(first_160_lats_expected) {
            assert_almost_eq!(actual, expected, delta);
        }

        // Results from the following command line using ecCodes:
        //
        // ```
        // xzcat testdata/gdas.t00z.sfluxgrbf000.grib2.0.xz \
        //     | grib_get_data -m foo -L "%11.6f%11.6f" - \
        //     | grep -v '^Latitude' | awk '{print $2;}' | head -160
        // ```
        let first_160_lons_expected = "
                0.000000  0.117188  0.234375  0.351563  0.468750  0.585938  0.703125  0.820313
                0.937500  1.054688  1.171875  1.289063  1.406250  1.523438  1.640625  1.757813
                1.875000  1.992188  2.109375  2.226563  2.343750  2.460938  2.578125  2.695313
                2.812500  2.929688  3.046875  3.164063  3.281250  3.398438  3.515625  3.632813
                3.750000  3.867188  3.984375  4.101563  4.218750  4.335938  4.453125  4.570313
                4.687500  4.804688  4.921875  5.039063  5.156250  5.273438  5.390625  5.507813
                5.625000  5.742188  5.859375  5.976563  6.093750  6.210938  6.328125  6.445313
                6.562500  6.679688  6.796875  6.914063  7.031250  7.148438  7.265625  7.382813
                7.500000  7.617188  7.734375  7.851563  7.968750  8.085938  8.203125  8.320313
                8.437500  8.554688  8.671875  8.789063  8.906250  9.023438  9.140625  9.257813
                9.375000  9.492188  9.609375  9.726563  9.843750  9.960938  10.078125 10.195313
                10.312500 10.429688 10.546875 10.664063 10.781250 10.898438 11.015625 11.132813
                11.250000 11.367188 11.484375 11.601563 11.718750 11.835938 11.953125 12.070313
                12.187500 12.304688 12.421875 12.539063 12.656250 12.773438 12.890625 13.007813
                13.125000 13.242188 13.359375 13.476563 13.593750 13.710938 13.828125 13.945313
                14.062500 14.179688 14.296875 14.414063 14.531250 14.648438 14.765625 14.882813
                15.000000 15.117188 15.234375 15.351563 15.468750 15.585938 15.703125 15.820313
                15.937500 16.054688 16.171875 16.289063 16.406250 16.523438 16.640625 16.757813
                16.875000 16.992188 17.109375 17.226563 17.343750 17.460938 17.578125 17.695313
                17.812500 17.929688 18.046875 18.164063 18.281250 18.398438 18.515625 18.632813
                ";
        let first_160_lons_expected = first_160_lons_expected
            .split_whitespace()
            .filter_map(|s| s.parse::<f32>().ok());

        let first_160_lons = first_submessage.latlons()?.map(|(_lat, lon)| lon).take(160);
        for (actual, expected) in first_160_lons.zip(first_160_lons_expected) {
            assert_almost_eq!(actual, expected, delta);
        }

        Ok(())
    }

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
