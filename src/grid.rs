use helpers::RegularGridIterator;

pub use self::{
    earth::EarthShapeDefinition,
    gaussian::{compute_gaussian_latitudes, GaussianGridDefinition},
    lambert::LambertGridDefinition,
    latlon::LatLonGridDefinition,
    polar_stereographic::PolarStereographicGridDefinition,
};

/// An iterator over latitudes and longitudes of grid points in a submessage.
///
/// This `enum` is created by the [`latlons`] method on [`SubMessage`]. See its
/// documentation for more.
///
/// [`latlons`]: crate::context::SubMessage::latlons
/// [`SubMessage`]: crate::context::SubMessage
#[derive(Clone)]
pub enum GridPointIterator {
    LatLon(RegularGridIterator),
    Lambert(std::vec::IntoIter<(f32, f32)>),
}

impl Iterator for GridPointIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::LatLon(iter) => iter.next(),
            Self::Lambert(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::LatLon(iter) => iter.size_hint(),
            Self::Lambert(iter) => iter.size_hint(),
        }
    }
}

/// An iterator over `(i, j)` of grid points.
///
/// This `struct` is created by the [`ij`] method. See its documentation for
/// more.
///
/// [`ij`]: LatLonGridDefinition::ij
#[derive(Clone)]
pub struct GridPointIndexIterator {
    major_len: usize,
    minor_len: usize,
    scanning_mode: ScanningMode,
    major_pos: usize,
    minor_pos: usize,
    increments: bool,
}

impl GridPointIndexIterator {
    pub(crate) fn new(i_len: usize, j_len: usize, scanning_mode: ScanningMode) -> Self {
        let (major_len, minor_len) = if scanning_mode.is_consecutive_for_i() {
            (j_len, i_len)
        } else {
            (i_len, j_len)
        };

        Self {
            major_len,
            minor_len,
            scanning_mode,
            minor_pos: 0,
            major_pos: 0,
            increments: true,
        }
    }
}

impl Iterator for GridPointIndexIterator {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.major_pos == self.major_len {
            return None;
        }

        let minor = if self.increments {
            self.minor_pos
        } else {
            self.minor_len - self.minor_pos - 1
        };
        let major = self.major_pos;

        self.minor_pos += 1;
        if self.minor_pos == self.minor_len {
            self.major_pos += 1;
            self.minor_pos = 0;
            if self.scanning_mode.scans_alternating_rows() {
                self.increments = !self.increments;
            }
        }

        if self.scanning_mode.is_consecutive_for_i() {
            Some((minor, major))
        } else {
            Some((major, minor))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.major_len - self.major_pos) * self.minor_len - self.minor_pos;
        (len, Some(len))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ScanningMode(pub u8);

impl ScanningMode {
    /// Returns `true` if points of the first row or column scan in the `+i`
    /// (`+x`) direction.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::ScanningMode(0b00000000).scans_positively_for_i(),
    ///     true
    /// );
    /// ```
    pub fn scans_positively_for_i(&self) -> bool {
        self.0 & 0b10000000 == 0
    }

    /// Returns `true` if points of the first row or column scan in the `+j`
    /// (`+y`) direction.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::ScanningMode(0b00000000).scans_positively_for_j(),
    ///     false
    /// );
    /// ```
    pub fn scans_positively_for_j(&self) -> bool {
        self.0 & 0b01000000 != 0
    }

    /// Returns `true` if adjacent points in `i` (`x`) direction are
    /// consecutive.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(grib::ScanningMode(0b00000000).is_consecutive_for_i(), true);
    /// ```
    pub fn is_consecutive_for_i(&self) -> bool {
        self.0 & 0b00100000 == 0
    }

    /// Returns `true` if adjacent rows scans in the opposite direction.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::ScanningMode(0b00000000).scans_alternating_rows(),
    ///     false
    /// );
    /// ```
    pub fn scans_alternating_rows(&self) -> bool {
        self.0 & 0b00010000 != 0
    }

    pub(crate) fn has_unsupported_flags(&self) -> bool {
        self.0 & 0b00001111 != 0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ProjectionCentreFlag(pub u8);

impl ProjectionCentreFlag {
    /// Returns `true` if North Pole is on the projection plane. Otherwise (i.e.
    /// if South Pole is on), returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::ProjectionCentreFlag(0b00000000).contains_north_pole_on_projection_plane(),
    ///     true
    /// );
    /// ```
    pub fn contains_north_pole_on_projection_plane(&self) -> bool {
        self.0 & 0b10000000 == 0
    }

    /// Returns `true` if projection is bipolar and symmetric. Otherwise (i.e.
    /// if only one projection centre is used), returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(grib::ProjectionCentreFlag(0b00000000).is_bipolar(), false);
    /// ```
    pub fn is_bipolar(&self) -> bool {
        self.0 & 0b01000000 != 0
    }

    #[allow(dead_code)]
    pub(crate) fn has_unsupported_flags(&self) -> bool {
        self.0 & 0b00111111 != 0
    }
}

mod earth;
mod gaussian;
mod helpers;
mod lambert;
mod latlon;
mod polar_stereographic;
