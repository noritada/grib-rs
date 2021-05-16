mod core;
pub use self::core::TableLookupResult::{self, Found, NotFound};
pub mod grib2;
mod old;
pub use old::*;
