use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

/// Product definition template 4.0 - Analysis or forecast at a horizontal level
/// or in a horizontal layer at a point in time.
#[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template4_0 {
    pub param: param_set::Parameter,
    pub generating_process: param_set::GeneratingProcessType,
    pub forecast_time: param_set::ForecastTime,
    pub horizontal: param_set::HorizontalSurfaces,
}

pub(crate) mod param_set {
    use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

    #[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct Parameter {
        /// Parameter category (see Code Table 4.1).
        pub category: u8,
        /// Parameter number (see Code Table 4.2).
        pub num: u8,
    }

    #[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct GeneratingProcessType {
        /// Type of generating process (see Code Table 4.3).
        pub process_type: u8,
        /// Background generating process identifier (defined by originating
        /// centre).
        pub background_process: u8,
        /// Analysis or forecast generating processes identifier (defined by
        /// originating centre).
        pub process_id: u8,
    }

    #[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct ForecastTime {
        /// Hours of observational data cutoff after reference time (see Note
        /// 1).
        pub cutoff_hours: u16,
        /// Minutes of observational data cutoff after reference time.
        pub cutoff_minutes: u8,
        /// Indicator of unit of time range (see Code Table 4.4).
        pub unit: u8,
        /// Forecast time in units defined by octet 18.
        pub time: i32,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct HorizontalSurfaces {
        pub first_surface: FixedSurface,
        pub second_surface: FixedSurface,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct FixedSurface {
        /// Type of fixed surface (see Code Table 4.5).
        pub surface_type: u8,
        /// Scale factor of fixed surface.
        pub scale_factor: i8,
        /// Scaled value of fixed surface.
        pub scaled_value: i32,
    }
}
