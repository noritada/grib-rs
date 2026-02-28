impl crate::def::grib2::template::param_set::ScanningMode {
    /// Returns `true` if points of the first row or column scan in the `+i`
    /// (`+x`) direction.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::def::grib2::template::param_set::ScanningMode(0b00000000).scans_positively_for_i(),
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
    ///     grib::def::grib2::template::param_set::ScanningMode(0b00000000).scans_positively_for_j(),
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
    /// assert_eq!(
    ///     grib::def::grib2::template::param_set::ScanningMode(0b00000000).is_consecutive_for_i(),
    ///     true
    /// );
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
    ///     grib::def::grib2::template::param_set::ScanningMode(0b00000000).scans_alternating_rows(),
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

impl crate::def::grib2::template::param_set::ProjectionCentreFlag {
    /// Returns `true` if North Pole is on the projection plane. Otherwise (i.e.
    /// if South Pole is on), returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     grib::def::grib2::template::param_set::ProjectionCentreFlag(0b00000000)
    ///         .contains_north_pole_on_projection_plane(),
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
    /// assert_eq!(
    ///     grib::def::grib2::template::param_set::ProjectionCentreFlag(0b00000000).is_bipolar(),
    ///     false
    /// );
    /// ```
    pub fn is_bipolar(&self) -> bool {
        self.0 & 0b01000000 != 0
    }

    #[allow(dead_code)]
    pub(crate) fn has_unsupported_flags(&self) -> bool {
        self.0 & 0b00111111 != 0
    }
}
