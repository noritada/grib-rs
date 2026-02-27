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
