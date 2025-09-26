use std::fmt::{self, Display, Formatter};

use crate::codetables::{grib2::*, *};

/// Parameter of the product.
///
/// In the context of GRIB products, parameters refer to weather elements such
/// as air temperature, air pressure, and humidity, and other physical
/// quantities.
///
/// With [`is_identical_to`], users can check if the parameter is identical to a
/// third-party code, such as [`NCEP`].
///
/// [`is_identical_to`]: Parameter::is_identical_to
#[derive(Debug, PartialEq, Eq)]
pub struct Parameter {
    /// Discipline of processed data in the GRIB message.
    pub discipline: u8,
    /// GRIB master tables version number.
    pub centre: u16,
    /// Parameter category by product discipline.
    pub master_ver: u8,
    /// GRIB local tables version number.
    pub local_ver: u8,
    /// Identification of originating/generating centre.
    pub category: u8,
    /// Parameter number by product discipline and parameter category.
    pub num: u8,
}

impl Parameter {
    /// Looks up the parameter's WMO description.
    ///
    /// # Examples
    ///
    /// ```
    /// // Extracted from the first submessage of JMA MSM GRIB2 data.
    /// let param = grib::Parameter {
    ///     discipline: 0,
    ///     centre: 34,
    ///     master_ver: 2,
    ///     local_ver: 1,
    ///     category: 3,
    ///     num: 5,
    /// };
    /// assert_eq!(param.description(), Some("Geopotential height".to_owned()))
    /// ```
    pub fn description(&self) -> Option<String> {
        CodeTable4_2::new(self.discipline, self.category)
            .lookup(usize::from(self.num))
            .description()
    }

    /// Checks if the parameter is identical to a third-party `code`, such as
    /// [`NCEP`].
    ///
    /// # Examples
    ///
    /// ```
    /// use grib::codetables::NCEP;
    ///
    /// // Extracted from the first submessage of JMA MSM GRIB2 data.
    /// let param = grib::Parameter {
    ///     discipline: 0,
    ///     centre: 34,
    ///     master_ver: 2,
    ///     local_ver: 1,
    ///     category: 3,
    ///     num: 5,
    /// };
    /// assert!(param.is_identical_to(NCEP::HGT));
    /// ```
    pub fn is_identical_to<'a, T>(&'a self, code: T) -> bool
    where
        T: TryFrom<&'a Self>,
        T: PartialEq,
    {
        let self_ = T::try_from(self);
        self_.is_ok_and(|v| v == code)
    }

    pub(crate) fn as_u32(&self) -> u32 {
        (u32::from(self.discipline) << 16) + (u32::from(self.category) << 8) + u32::from(self.num)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ForecastTime {
    pub unit: Code<grib2::Table4_4, u8>,
    pub value: u32,
}

impl ForecastTime {
    pub fn new(unit: Code<grib2::Table4_4, u8>, value: u32) -> Self {
        Self { unit, value }
    }

    pub fn from_numbers(unit: u8, value: u32) -> Self {
        let unit = Table4_4::try_from(unit).into();
        Self { unit, value }
    }

    pub fn describe(&self) -> (String, String) {
        let unit = match &self.unit {
            Name(unit) => format!("{unit:#?}"),
            Num(num) => format!("code {num:#?}"),
        };
        let value = self.value.to_string();
        (unit, value)
    }
}

impl Display for ForecastTime {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.value)?;

        match &self.unit {
            Name(unit) => {
                if let Some(expr) = unit.short_expr() {
                    write!(f, " [{expr}]")?;
                }
            }
            Num(num) => {
                write!(f, " [unit: {num}]")?;
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

    /// Returns the unit string defined for the type of the surface, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(grib::FixedSurface::new(100, 0, 0).unit(), Some("Pa"));
    /// ```
    pub fn unit(&self) -> Option<&str> {
        // Tentative implementation; pattern matching should be generated from the
        // CodeFlag CSV file.
        let unit = match self.surface_type {
            11 => "m",
            12 => "m",
            13 => "%",
            18 => "Pa",
            20 => "K",
            21 => "kg m-3",
            22 => "kg m-3",
            23 => "Bq m-3",
            24 => "Bq m-3",
            25 => "dBZ",
            26 => "m",
            27 => "m",
            30 => "m",
            100 => "Pa",
            102 => "m",
            103 => "m",
            104 => r#""sigma" value"#,
            106 => "m",
            107 => "K",
            108 => "Pa",
            109 => "K m2 kg-1 s-1",
            114 => "Numeric",
            117 => "m",
            151 => "Numeric",
            152 => "Numeric",
            160 => "m",
            161 => "m",
            168 => "Numeric",
            169 => "kg m-3",
            170 => "K",
            171 => "m2 s-1",
            _ => return None,
        };
        Some(unit)
    }

    /// Checks if the scale factor should be treated as missing.
    pub fn scale_factor_is_nan(&self) -> bool {
        // Handle as NaN if all bits are 1. Note that this is i8::MIN + 1 and not
        // i8::MIN.
        self.scale_factor == i8::MIN + 1
    }

    /// Checks if the scaled value should be treated as missing.
    pub fn value_is_nan(&self) -> bool {
        // Handle as NaN if all bits are 1. Note that this is i32::MIN + 1 and not
        // i32::MIN.
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
