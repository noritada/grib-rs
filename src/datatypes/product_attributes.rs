use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use crate::codetables::grib2::*;
use crate::codetables::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ForecastTime {
    pub unit: TableLookupResult<grib2::Table4_4, u8>,
    pub value: u32,
}

impl ForecastTime {
    pub fn new(unit: u8, value: u32) -> Self {
        let unit = Table4_4::try_from(unit).into();
        Self { unit, value }
    }

    pub fn describe(&self) -> (String, String) {
        let unit = match &self.unit {
            Found(unit) => format!("{:#?}", unit),
            NotFound(num) => format!("code {:#?}", num),
        };
        let value = self.value.to_string();
        (unit, value)
    }
}

impl Display for ForecastTime {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.value)?;

        match &self.unit {
            Found(unit) => {
                if let Some(expr) = unit.short_expr() {
                    write!(f, " [{}]", expr)?;
                }
            }
            NotFound(num) => {
                write!(f, " [unit: {}]", num)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FixedSurface {
    /// Use [CodeTable4_5] to get textual representation.
    pub surface_type: u8,
    pub scale_factor: i8,
    pub scaled_value: i32,
}

impl FixedSurface {
    pub fn new(surface_type: u8, scale_factor: i8, scaled_value: i32) -> Self {
        Self {
            surface_type,
            scale_factor,
            scaled_value,
        }
    }

    pub fn value(&self) -> f64 {
        if self.value_is_nan() {
            f64::NAN
        } else {
            let factor: f64 = 10_f64.powi(-i32::from(self.scale_factor));
            f64::from(self.scaled_value) * factor
        }
    }

    /// Checks if the scale factor should be treated as missing.
    pub fn scale_factor_is_nan(&self) -> bool {
        // Handle as NaN if all bits are 1. Note that this is i8::MIN + 1 and not i8::MIN.
        self.scale_factor == i8::MIN + 1
    }

    /// Checks if the scaled value should be treated as missing.
    pub fn value_is_nan(&self) -> bool {
        // Handle as NaN if all bits are 1. Note that this is i32::MIN + 1 and not i32::MIN.
        self.scaled_value == i32::MIN + 1
    }

    pub fn describe(&self) -> (String, String, String) {
        let stype = CodeTable4_5
            .lookup(usize::from(self.surface_type))
            .to_string();
        let scale_factor = if self.scale_factor_is_nan() {
            "Missing".to_owned()
        } else {
            self.scale_factor.to_string()
        };
        let scaled_value = if self.value_is_nan() {
            "Missing".to_owned()
        } else {
            self.scaled_value.to_string()
        };
        (stype, scale_factor, scaled_value)
    }
}
