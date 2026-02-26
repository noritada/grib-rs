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
