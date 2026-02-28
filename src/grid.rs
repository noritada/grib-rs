use helpers::RegularGridIterator;

pub use self::{gaussian::compute_gaussian_latitudes, rotated_ll::Unrotate};
use crate::def::grib2::template::param_set::ScanningMode;

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
    RotatedLatLon(Unrotate<RegularGridIterator>),
    Lambert(std::vec::IntoIter<(f32, f32)>),
}

impl Iterator for GridPointIterator {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::LatLon(iter) => iter.next(),
            Self::RotatedLatLon(iter) => iter.next(),
            Self::Lambert(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::LatLon(iter) => iter.size_hint(),
            Self::RotatedLatLon(iter) => iter.size_hint(),
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

mod earth;
mod flags;
mod gaussian;
mod helpers;
mod lambert;
mod latlon;
mod polar_stereographic;
mod rotated_ll;
