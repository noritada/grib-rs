//! A collection of simple trait implementations for types defined in `src/def`.

use crate::def::grib2::table::{
    DataRepresentationTemplateNum, GridDefinitionTemplateNum, IdentificationTemplateNum,
    ProductDefinitionTemplateNum,
};

impl From<IdentificationTemplateNum> for u64 {
    fn from(value: IdentificationTemplateNum) -> Self {
        u64::from(value.0)
    }
}

impl From<GridDefinitionTemplateNum> for u64 {
    fn from(value: GridDefinitionTemplateNum) -> Self {
        u64::from(value.0)
    }
}

impl From<ProductDefinitionTemplateNum> for u64 {
    fn from(value: ProductDefinitionTemplateNum) -> Self {
        u64::from(value.0)
    }
}

impl From<DataRepresentationTemplateNum> for u64 {
    fn from(value: DataRepresentationTemplateNum) -> Self {
        u64::from(value.0)
    }
}
