use grib_template_derive::{Dump, TryFromSlice};

/// Grid definition template 3.0 - latitude/longitude (or equidistant
/// cylindrical, or Plate Carrée).
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template3_0 {
    pub earth: param_set::EarthShape,
    pub lat_lon: param_set::LatLonGrid,
}

/// Grid definition template 3.1 - rotated latitude/longitude (or equidistant
/// cylindrical, or Plate Carrée).
///
/// # Examples
///
/// ```
/// use std::io::Read;
///
/// use grib::def::grib2::template::Template3_1;
/// use grib_template_helpers::TryFromSlice;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut buf = Vec::new();
///
///     let f = std::fs::File::open(
///         "testdata/20260219T00Z_MSC_HRDPS_CAPE_Sfc_RLatLon0.0225_PT000H.grib2",
///     )?;
///     let mut f = std::io::BufReader::new(f);
///     f.read_to_end(&mut buf)?;
///
///     let mut pos = 0x33;
///     let actual = Template3_1::try_from_slice(&buf, &mut pos)?;
///     let expected = Template3_1 {
///         earth: grib::def::grib2::template::param_set::EarthShape {
///             shape: 6,
///             spherical_earth_radius_scale_factor: 0xff,
///             spherical_earth_radius_scaled_value: 0xffffffff,
///             major_axis_scale_factor: 0xff,
///             major_axis_scaled_value: 0xffffffff,
///             minor_axis_scale_factor: 0xff,
///             minor_axis_scaled_value: 0xffffffff,
///         },
///         rotated: grib::def::grib2::template::param_set::LatLonGrid {
///             grid: grib::def::grib2::template::param_set::Grid {
///                 ni: 2540,
///                 nj: 1290,
///                 initial_production_domain_basic_angle: 0,
///                 basic_angle_subdivisions: 0xffffffff,
///                 first_point_lat: -12302501,
///                 first_point_lon: 345178780,
///                 resolution_and_component_flags:
///                     grib::def::grib2::template::param_set::ResolutionAndComponentFlags(
///                         0b00111000,
///                     ),
///                 last_point_lat: 16700001,
///                 last_point_lon: 42306283,
///             },
///             scanning_mode: grib::def::grib2::template::param_set::ScanningMode(0b01000000),
///             i_direction_inc: 22500,
///             j_direction_inc: 22500,
///         },
///         rotation: grib::def::grib2::template::param_set::Rotation {
///             south_pole_lat: -36088520,
///             south_pole_lon: 245305142,
///             rot_angle: 0.,
///         },
///     };
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template3_1 {
    pub earth: param_set::EarthShape,
    pub rotated: param_set::LatLonGrid,
    pub rotation: param_set::Rotation,
}

/// Grid definition template 3.20 - polar stereographic projection.
///
/// # Examples
///
/// ```
/// use std::io::{BufReader, Read};
///
/// use grib::def::grib2::template::Template3_20;
/// use grib_template_helpers::TryFromSlice;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut buf = Vec::new();
///
///     let f = std::fs::File::open(
///         "testdata/CMC_RDPA_APCP-024-0100cutoff_SFC_0_ps10km_2023121806_000.grib2.xz",
///     )?;
///     let f = BufReader::new(f);
///     let mut f = xz2::bufread::XzDecoder::new(f);
///     f.read_to_end(&mut buf)?;
///
///     let mut pos = 0x33;
///     let actual = Template3_20::try_from_slice(&buf, &mut pos)?;
///     let expected = Template3_20 {
///         earth_shape: grib::def::grib2::template::param_set::EarthShape {
///             shape: 6,
///             spherical_earth_radius_scale_factor: 0xff,
///             spherical_earth_radius_scaled_value: 0xffffffff,
///             major_axis_scale_factor: 0xff,
///             major_axis_scaled_value: 0xffffffff,
///             minor_axis_scale_factor: 0xff,
///             minor_axis_scaled_value: 0xffffffff,
///         },
///         ni: 935,
///         nj: 824,
///         first_point_lat: 18145030,
///         first_point_lon: 217107456,
///         resolution_and_component_flags:
///             grib::def::grib2::template::param_set::ResolutionAndComponentFlags(0b00001000),
///         lad: 60000000,
///         lov: 249000000,
///         dx: 10000000,
///         dy: 10000000,
///         projection_centre: grib::def::grib2::template::param_set::ProjectionCentreFlag(
///             0b00000000,
///         ),
///         scanning_mode: grib::def::grib2::template::param_set::ScanningMode(0b01000000),
///     };
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct Template3_20 {
    pub earth_shape: param_set::EarthShape,
    /// Nx - number of points along the x-axis.
    pub ni: u32,
    /// Ny - number of points along the y-axis.
    pub nj: u32,
    /// La1 - latitude of first grid point.
    pub first_point_lat: i32,
    /// Lo1 - longitude of first grid point.
    pub first_point_lon: i32,
    pub resolution_and_component_flags: param_set::ResolutionAndComponentFlags,
    /// LaD - latitude where Dx and Dy are specified.
    pub lad: i32,
    /// LoV - orientation of the grid (see Note 2).
    pub lov: i32,
    /// Dx - x-direction grid length (see Note 3).
    pub dx: u32,
    /// Dy - y-direction grid length (see Note 3).
    pub dy: u32,
    pub projection_centre: param_set::ProjectionCentreFlag,
    pub scanning_mode: param_set::ScanningMode,
}

/// Grid definition template 3.30 - Lambert conformal.
///
/// # Examples
///
/// ```
/// use std::io::{BufReader, Read};
///
/// use grib::def::grib2::template::Template3_30;
/// use grib_template_helpers::TryFromSlice;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut buf = Vec::new();
///
///     let f = std::fs::File::open("testdata/ds.critfireo.bin.xz")?;
///     let f = BufReader::new(f);
///     let mut f = xz2::bufread::XzDecoder::new(f);
///     f.read_to_end(&mut buf)?;
///
///     let mut pos = 0x83;
///     let actual = Template3_30::try_from_slice(&buf, &mut pos)?;
///     let expected = Template3_30 {
///         earth_shape: grib::def::grib2::template::param_set::EarthShape {
///             shape: 1,
///             spherical_earth_radius_scale_factor: 0,
///             spherical_earth_radius_scaled_value: 6371200,
///             major_axis_scale_factor: 0,
///             major_axis_scaled_value: 0,
///             minor_axis_scale_factor: 0,
///             minor_axis_scaled_value: 0,
///         },
///         ni: 2145,
///         nj: 1377,
///         first_point_lat: 20190000,
///         first_point_lon: 238449996,
///         resolution_and_component_flags:
///             grib::def::grib2::template::param_set::ResolutionAndComponentFlags(0b00000000),
///         lad: 25000000,
///         lov: 265000000,
///         dx: 2539703,
///         dy: 2539703,
///         projection_centre: grib::def::grib2::template::param_set::ProjectionCentreFlag(
///             0b00000000,
///         ),
///         scanning_mode: grib::def::grib2::template::param_set::ScanningMode(0b01010000),
///         latin1: 25000000,
///         latin2: 25000000,
///         south_pole_lat: -90000000,
///         south_pole_lon: 0,
///     };
///     assert_eq!(actual, expected);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
pub struct Template3_30 {
    pub earth_shape: param_set::EarthShape,
    /// Nx - number of points along the x-axis.
    pub ni: u32,
    /// Ny - number of points along the y-axis.
    pub nj: u32,
    /// La1 - latitude of first grid point.
    pub first_point_lat: i32,
    /// Lo1 - longitude of first grid point.
    pub first_point_lon: i32,
    pub resolution_and_component_flags: param_set::ResolutionAndComponentFlags,
    /// LaD - latitude where Dx and Dy are specified.
    pub lad: i32,
    /// LoV - longitude of meridian parallel to y-axis along which latitude
    /// increases as the y-coordinate increases.
    pub lov: i32,
    /// Dx - x-direction grid length (see Note 1).
    pub dx: u32,
    /// Dy - y-direction grid length (see Note 1).
    pub dy: u32,
    pub projection_centre: param_set::ProjectionCentreFlag,
    pub scanning_mode: param_set::ScanningMode,
    /// Latin 1 - first latitude from the pole at which the secant cone cuts the
    /// sphere.
    pub latin1: i32,
    /// Latin 2 - second latitude from the pole at which the secant cone cuts
    /// the sphere.
    pub latin2: i32,
    /// Latitude of the southern pole of projection.
    pub south_pole_lat: i32,
    /// Longitude of the southern pole of projection.
    pub south_pole_lon: i32,
}

/// Grid definition template 3.40 - Gaussian latitude/longitude.
#[derive(Debug, PartialEq, TryFromSlice, Dump)]
pub struct Template3_40 {
    pub earth: param_set::EarthShape,
    pub gaussian: param_set::GaussianGrid,
}

pub(crate) mod param_set {
    use grib_template_derive::{Dump, TryFromSlice};

    #[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
    pub struct EarthShape {
        /// Shape of the Earth (see Code table 3.2).
        pub shape: u8,
        /// Scale factor of radius of spherical Earth.
        pub spherical_earth_radius_scale_factor: u8,
        /// Scaled value of radius of spherical Earth.
        pub spherical_earth_radius_scaled_value: u32,
        /// Scale factor of major axis of oblate spheroid Earth.
        pub major_axis_scale_factor: u8,
        /// Scaled value of major axis of oblate spheroid Earth.
        pub major_axis_scaled_value: u32,
        /// Scale factor of minor axis of oblate spheroid Earth.
        pub minor_axis_scale_factor: u8,
        /// Scaled value of minor axis of oblate spheroid Earth.
        pub minor_axis_scaled_value: u32,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
    pub struct LatLonGrid {
        pub grid: Grid,
        /// Di - i direction increment (see Notes 1 and 5).
        pub i_direction_inc: u32,
        /// Dj - j direction increment (see Notes 1 and 5).
        pub j_direction_inc: u32,
        pub scanning_mode: ScanningMode,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
    pub struct GaussianGrid {
        pub grid: Grid,
        /// Di - i direction increment (see Notes 1 and 5).
        pub i_direction_inc: u32,
        /// N - number of parallels between a pole and the Equator (see Note 2).
        pub n: u32,
        pub scanning_mode: ScanningMode,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, Dump)]
    pub struct Grid {
        /// Ni - number of points along a parallel.
        pub ni: u32,
        /// Nj - number of points along a meridian.
        pub nj: u32,
        /// Basic angle of the initial production domain (see Note 1).
        pub initial_production_domain_basic_angle: u32,
        /// Subdivisions of basic angle used to define extreme longitudes and
        /// latitudes, and direction increments (see Note 1).
        pub basic_angle_subdivisions: u32,
        /// La1 - latitude of first grid point (see Note 1).
        pub first_point_lat: i32,
        /// Lo1 - longitude of first grid point (see Note 1).
        pub first_point_lon: i32,
        pub resolution_and_component_flags: ResolutionAndComponentFlags,
        /// La2 - latitude of last grid point (see Note 1).
        pub last_point_lat: i32,
        /// Lo2 - longitude of last grid point (see Note 1).
        pub last_point_lon: i32,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, Dump)]
    pub struct ProjectionCentreFlag(
        /// Projection centre flag (see Flag table 3.5).
        pub u8,
    );

    #[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, Dump)]
    pub struct ResolutionAndComponentFlags(
        /// Resolution and component flags (see Flag table 3.3).
        pub u8,
    );

    #[derive(Debug, PartialEq, TryFromSlice, Dump)]
    pub struct Rotation {
        /// Latitude of the southern pole of projection.
        pub south_pole_lat: i32,
        /// Longitude of the southern pole of projection.
        pub south_pole_lon: i32,
        /// Angle of rotation of projection.
        pub rot_angle: f32,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, Dump)]
    pub struct ScanningMode(
        /// Scanning mode (flags - see Flag table 3.4).
        pub u8,
    );
}
