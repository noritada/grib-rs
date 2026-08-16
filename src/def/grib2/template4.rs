use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

/// Product definition template 4.0 - Analysis or forecast at a horizontal level
/// or in a horizontal layer at a point in time.
#[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template4_0 {
    pub param: param_set::ProductParam,
    pub generating_process: param_set::GeneratingProcess,
    #[dump(doc(
        cutoff_hours = "Hours of observational data cutoff after reference time (see Note 1)",
        cutoff_minutes = "Minutes of observational data cutoff after reference time",
    ))]
    pub forecast_time: param_set::ForecastTime,
    pub horizontal: param_set::Horizontal,
}

/// Product definition template 4.1 - Individual ensemble forecast, control and
/// perturbed, at a horizontal level or in a horizontal layer at a point in
/// time.
#[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template4_1 {
    pub param: param_set::ProductParam,
    #[dump(doc(
        process_id = "Forecast generating process identifier (defined by originating centre).",
    ))]
    pub generating_process: param_set::GeneratingProcess,
    pub forecast_time: param_set::ForecastTime,
    pub horizontal: param_set::Horizontal,
    pub emsemble_forecast: param_set::EnsembleForecast,
}

/// Product definition template 4.2 - Derived forecast based on all ensemble
/// members at a horizontal level or in a horizontal layer at a point in time.
#[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template4_2 {
    pub param: param_set::ProductParam,
    #[dump(doc(
        process_id = "Forecast generating process identifier (defined by originating centre).",
    ))]
    pub generating_process: param_set::GeneratingProcess,
    pub forecast_time: param_set::ForecastTime,
    pub horizontal: param_set::Horizontal,
    pub derived_forecast: param_set::DerivedForecast,
}

/// Product definition template 4.5 - Probability forecasts at a horizontal
/// level or in a horizontal layer at a point in time.
#[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template4_5 {
    pub param: param_set::ProductParam,
    #[dump(doc(
        process_id = "Forecast generating process identifier (defined by originating centre).",
    ))]
    pub generating_process: param_set::GeneratingProcess,
    pub forecast_time: param_set::ForecastTime,
    pub horizontal: param_set::Horizontal,
    pub probability_forecasts: param_set::ProbabilityForecasts,
}

/// Product definition template 4.6 - Percentile forecasts at a horizontal level
/// or in a horizontal layer at a point in time.
#[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
pub struct Template4_6 {
    pub param: param_set::ProductParam,
    pub generating_process: param_set::GeneratingProcess,
    pub forecast_time: param_set::ForecastTime,
    pub horizontal: param_set::Horizontal,
    pub percentile_forecasts: param_set::PercentileForecasts,
}

pub(crate) mod param_set {
    use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

    use super::super::template::param_set::{TimeRange, ValueWithScaling};

    #[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct ProductParam {
        /// Parameter category (see Code Table 4.1).
        pub category: u8,
        /// Parameter number (see Code Table 4.2).
        pub num: u8,
    }

    #[derive(Debug, PartialEq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct GeneratingProcess {
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
        /// Hours after reference time of data cutoff (see Note 1).
        pub cutoff_hours: u16,
        /// Minutes after reference time of data cutoff.
        pub cutoff_minutes: u8,
        /// Forecast time.
        #[dump(doc(
            unit = "Indicator of unit of time range (see Code Table 4.4).",
            len = "Forecast time in units defined by octet 18.",
        ))]
        pub value: TimeRange<i32>,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct Horizontal {
        #[dump(doc(
            surface_type = "Type of first fixed surface (see Code Table 4.5).",
            scale_factor = "Scale factor of first fixed surface.",
            scaled_value = "Scaled value of first fixed surface."
        ))]
        pub first_surface: FixedSurface,
        #[dump(doc(
            surface_type = "Type of second fixed surface (see Code Table 4.5).",
            scale_factor = "Scale factor of second fixed surface.",
            scaled_value = "Scaled value of second fixed surface."
        ))]
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

    #[derive(Debug, PartialEq, Eq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct EnsembleForecast {
        /// Type of ensemble forecast (see Code Table 4.6).
        pub ensemble_forecast_type: u8,
        ///  Perturbation number.
        pub perturbation_num: u8,
        /// Number of forecasts in ensemble.
        pub num_forecasts: u8,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct DerivedForecast {
        /// Derived forecast (see Code Table 4.7).
        pub derived_forecast_type: u8,
        /// Number of forecasts in ensemble.
        pub num_forecasts: u8,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct ProbabilityForecasts {
        /// Forecast probability number.
        pub forecast_probability_num: u8,
        /// Total number of forecast probabilities.
        pub total_num_forecast_probabilities: u8,
        /// Probability type (see Code Table 4.9).
        pub probability_type: u8,
        /// Lower limit.
        #[dump(doc(
            scale_factor = "Scale factor of lower limit.",
            scaled_value = "Scaled value of lower limit.",
        ))]
        pub lower_limit: ValueWithScaling<i8, i32>,
        /// Upper limit.
        #[dump(doc(
            scale_factor = "Scale factor of upper limit.",
            scaled_value = "Scaled value of upper limit.",
        ))]
        pub upper_limit: ValueWithScaling<i8, i32>,
    }

    #[derive(Debug, PartialEq, Eq, TryFromSlice, WriteToBuffer, Dump)]
    pub struct PercentileForecasts {
        /// Percentile value (from 100% to 0%).
        pub percentile_value: u8,
    }
}
