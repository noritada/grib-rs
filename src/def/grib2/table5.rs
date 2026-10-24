use grib_template_derive::{Dump, TryFromSlice, WriteToBuffer};

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct DataRepresentationTemplateNum(
    /// Data representation template number (see Code table 5.0).
    pub u16,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct OriginalFieldValueType(
    /// Type of original field values (see Code table 5.1).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct MatrixCoordinateValueFuncDefinition(
    /// Coordinate value definition (Code table 5.2).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct MatrixCoordinateParam(
    /// Physical significance (Code table 5.3).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct GroupSplittingMethod(
    /// Group splitting method used (see Code table 5.4).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct ComplexPackingMissingValueManagement(
    /// Missing value management used (see Code table 5.5).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct SpatialDifferencingOrder(
    /// Order of spatial differencing (see Code table 5.6).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct FloatPrecision(
    /// Precision (see Code table 5.7).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct BiFourierSubtruncationType(
    /// Bi-Fourier sub-truncation type (see Code table 5.25).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct BiFourierPackMode(
    /// Packing mode for axes (see Code table 5.26).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
pub struct CompressionType(
    /// Type of compression used (see Code table 5.40).
    pub u8,
);

#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromSlice, WriteToBuffer, Dump)]
#[dump(flag)]
pub struct OrderingFlags(
    /// Ordering flags (Flag table 5.50002).
    pub u8,
);
