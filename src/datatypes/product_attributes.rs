use crate::codetables::*;

#[derive(Debug, PartialEq, Eq)]
pub struct ForecastTime {
    /// Use [CodeTable4_4] to get textual representation.
    pub unit: u8,
    pub value: u32,
}

impl ForecastTime {
    pub fn new(unit: u8, value: u32) -> Self {
        Self { unit, value }
    }

    pub fn describe(&self) -> (String, String) {
        let unit = CodeTable4_4.lookup(usize::from(self.unit)).to_string();
        let value = self.value.to_string();
        (unit, value)
    }
}

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
