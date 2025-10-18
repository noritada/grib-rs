use grib_template_derive::{Dump, TryFromSlice};

/// Data representation template 5.0 - Grid point data - simple packing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_0 {
    pub simple: param_set::SimplePacking,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
}

/// Data representation template 5.1 - Matrix value at grid point - simple
/// packing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_1 {
    pub simple: param_set::SimplePacking,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
    /// 0, no matrix bit maps present; 1-matrix bit maps present.
    pub matrix_bitmap_present: u8,
    /// Number of data values encoded in Section 7.
    pub num_encoded_vals: u32,
    /// NR - first dimension (rows) of each matrix.
    pub num_dim_1: u16,
    /// NC - second dimension (columns) of each matrix.
    pub num_dim_2: u16,
    /// First dimension coordinate value definition (Code table 5.2).
    pub dim_1_coord_def: u8,
    /// NC1 - number of coefficients or values used to specify first dimension
    /// coordinate function.
    pub num_dim_1_coeffs: u8,
    /// Second dimension coordinate value definition (Code table 5.2).
    pub dim_2_coord_def: u8,
    /// NC2 - number of coefficients or values used to specify second dimension
    /// coordinate function.
    pub num_dim_2_coeffs: u8,
    /// First dimension physical significance (Code table 5.3).
    pub dim_1_significance: u8,
    /// Second dimension physical significance (Code table 5.3).
    pub dim_2_significance: u8,
    /// Coefficients to define first dimension coordinate values in functional
    /// form, or the explicit coordinate values (IEEE 32-bit floating-point
    /// value).
    #[grib_template(len = "num_dim_1_coeffs")]
    pub dim_1_coeffs: Vec<f32>,
    /// Coefficients to define second dimension coordinate values in functional
    /// form, or the explicit coordinate values (IEEE 32-bit floating-point
    /// value).
    #[grib_template(len = "num_dim_2_coeffs")]
    pub dim_2_coeffs: Vec<f32>,
}

/// Data representation template 5.2 - Grid point data - complex packing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_2 {
    pub simple: param_set::SimplePacking,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
    pub complex: param_set::ComplexPacking,
}

/// Data representation template 5.3 - Grid point data - complex packing and
/// spatial differencing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_3 {
    pub simple: param_set::SimplePacking,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
    pub complex: param_set::ComplexPacking,
    /// Order of spatial differencing (see Code table 5.6).
    pub spatial_diff_order: u8,
    /// Number of octets required in the data section to specify extra
    /// descriptors needed for spatial differencing (octets 6-ww in data
    /// template 7.3).
    pub num_extra_desc_octets: u8,
}

/// Data representation template 5.4 - Grid point data - IEEE floating point
/// data.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_4 {
    /// Precision (see Code table 5.7).
    pub precision: u8,
}

/// Data representation template 5.40 - Grid point data - JPEG 2000 code stream
/// format.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_40 {
    pub simple: param_set::SimplePacking,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
    /// Type of compression used (see Code table 5.40).
    pub compression_type: u8,
    /// Target compression ratio, M:1 (with respect to the bit-depth specified
    /// in octet 20), when octet 22 indicates lossy compression. Otherwise, set
    /// to missing (see Note 3).
    pub compression_ratio: u8,
}

/// Data representation template 5.41 - Grid point data - Portable Network
/// Graphics (PNG).
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_41 {
    pub simple: param_set::SimplePacking,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
}

/// Data representation template 5.42 - Grid point data - CCSDS recommended
/// lossless compression.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_42 {
    pub simple: param_set::SimplePacking,
    /// Type of original field values (see Code table 5.1).
    pub orig_field_type: u8,
    /// CCSDS compression options mask (see Note 3).
    pub mask: u8,
    /// Block size.
    pub block_size: u8,
    /// Reference sample interval.
    pub ref_sample_interval: u16,
}

/// Data representation template 5.50 - Spectral data - simple packing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_50 {
    pub simple: param_set::SimplePacking,
    /// Real part of (0.0) coefficient (IEEE 32-bit floating-point value).
    pub real_part_zero: f32,
}

/// Data representation template 5.51 - Spherical harmonics data - complex
/// packing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_51 {
    pub simple: param_set::SimplePacking,
    /// P - Laplacian scaling factor (expressed in 10-6 units).
    pub p: i32,
    /// JS - pentagonal resolution parameter of the unpacked subset (see Note
    /// 1).
    pub js: u16,
    /// KS - pentagonal resolution parameter of the unpacked subset (see Note
    /// 1).
    pub ks: u16,
    /// MS - pentagonal resolution parameter of the unpacked subset (see Note
    /// 1).
    pub ms: u16,
    /// TS - total number of values in the unpacked subset (see Note 1).
    pub ts: u32,
    /// Precision of the unpacked subset (see Code table 5.7).
    pub precision: u8,
}

/// Data representation template 5.53 - Spectral data for limited area models -
/// complex packing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_53 {
    pub simple: param_set::SimplePacking,
    /// Bi-Fourier sub-truncation type (see Code table 5.25).
    pub bi_fourier_subtrunc_type: u8,
    /// Packing mode for axes (see Code table 5.26).
    pub bi_fourier_pack_mode: u8,
    /// P - Laplacian scaling factor (expressed in 10-6 units).
    pub p: i32,
    /// NS - bi-Fourier resolution parameter of the unpacked subset (see Note
    /// 1).
    pub ns: u16,
    /// MS - bi-Fourier resolution parameter of the unpacked subset (see Note
    /// 1).
    pub ms: u16,
    /// TS - total number of values in the unpacked subset (see Note 1).
    pub ts: u32,
    /// Precision of the unpacked subset (see Code table 5.7).
    pub precision: u8,
}

/// Data representation template 5.61 - Grid point data - simple packing with
/// logarithm pre-processing.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template5_61 {
    pub simple: param_set::SimplePacking,
    /// Pre-processing parameter (B) (IEEE 32-bit floating-point value).
    pub preprocess_param: f32,
}

/// Data representation template 5.200 - Run length packing with level values.
#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct Template5_200 {
    /// Number of bits used for each packed value in the run length packing with
    /// level value.
    pub num_bits: u8,
    /// MV - maximum value within the levels that are used in the packing.
    pub max_val: u16,
    /// MVL - maximum value of level (predefined).
    pub max_level: u16,
    /// Decimal scale factor of representative value of each level.
    pub dec: u8,
    /// List of MVL scaled representative values of each level from lv=1 to MVL.
    #[grib_template(len = "max_level")]
    pub level_vals: Vec<u16>,
}

pub(crate) mod param_set {
    use grib_template_derive::{Dump, TryFromSlice};

    #[derive(Debug, PartialEq, TryFromSlice, Dump)]
    pub struct SimplePacking {
        /// Reference value (R) (IEEE 32-bit floating-point value).
        pub ref_val: f32,
        /// Binary scale factor (E).
        pub exp: i16,
        /// Decimal scale factor (D).
        pub dec: i16,
        /// Number of bits used for each packed value for simple packing, or for
        /// each group reference value for complex packing or spatial
        /// differencing.
        pub num_bits: u8,
    }

    impl SimplePacking {
        pub(crate) fn zero_bit_reference_value(&self) -> f32 {
            self.ref_val * 10_f32.powi(-i32::from(self.dec))
        }
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
    pub struct ComplexPacking {
        /// Group splitting method used (see Code table 5.4).
        pub group_splitting_method: u8,
        /// Missing value management used (see Code table 5.5).
        pub missing_value_management: u8,
        /// Primary missing value substitute.
        pub primary_missing_value: u32,
        /// Secondary missing value substitute.
        pub secondary_missing_value: u32,
        /// NG - number of groups of data values into which field is split.
        pub num_groups: u32,
        /// Reference for group widths (see Note 12).
        pub group_width_ref: u8,
        /// Number of bits used for the group widths (after the reference value
        /// in octet 36 has been removed).
        pub num_group_width_bits: u8,
        /// Reference for group lengths (see Note 13).
        pub group_len_ref: u32,
        /// Length increment for the group lengths (see Note 14).
        pub group_len_inc: u8,
        /// True length of last group.
        pub group_len_last: u32,
        /// Number of bits used for the scaled group lengths (after subtraction
        /// of the reference value given in octets 38-41 and division by
        /// the length increment given in octet 42).
        pub num_group_len_bits: u8,
    }
}
