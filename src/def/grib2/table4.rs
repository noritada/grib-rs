use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct ParamCategory(
    /// Parameter category (see Code Table 4.1).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct ParamNum(
    /// Parameter number (see Code Table 4.2).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct GeneratingProcessType(
    /// Type of generating process (see Code Table 4.3).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct TimeRangeUnit(
    /// Unit of time range (see Code Table 4.4).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct FixedSurfaceType(
    /// Type of fixed surface (see Code Table 4.5).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct EnsembleForecastType(
    /// Type of ensemble forecast (see Code Table 4.6).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct DerivedForecastType(
    /// Derived forecast (see Code Table 4.7).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct ProbabilityForecastType(
    /// Probability type (see Code Table 4.9).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct StatisticalProcessType(
    /// Statistical process used to calculate the processed field from the
    /// field at each time increment during the time range (see Code Table
    /// 4.10).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct TimeIncrementType(
    /// Type of time increment between successive fields used in the
    /// statistical processing (see Code Table 4.11).
    pub u8,
);
