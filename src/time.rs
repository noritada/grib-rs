use std::fmt;

#[cfg(feature = "chrono")]
use chrono::{DateTime, LocalResult, TimeZone, Utc};

#[cfg(feature = "chrono")]
use crate::error::GribError;

#[derive(Debug, PartialEq, Eq)]
pub struct UtcDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl UtcDateTime {
    pub fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

impl fmt::Display for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

#[cfg(feature = "chrono")]
#[inline]
pub(crate) fn create_date_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<DateTime<Utc>, GribError> {
    let result = Utc.with_ymd_and_hms(year, month, day, hour, minute, second);
    if let LocalResult::None = result {
        Err(GribError::InvalidValueError(format!(
            "invalid date time: {year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"
        )))
    } else {
        Ok(result.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_date_time_creation {
        ($((
            $name:ident,
            $year:expr,
            $month:expr,
            $day:expr,
            $hour:expr,
            $minute:expr,
            $second:expr,
            $ok_expected:expr
        ),)*) => ($(
            #[cfg(feature = "chrono")]
            #[test]
            fn $name() {
                let result = create_date_time($year, $month, $day, $hour, $minute, $second);
                assert_eq!(result.is_ok(), $ok_expected);
            }
        )*);
    }

    test_date_time_creation! {
        (date_time_creation_for_valid_date_time, 2022, 1, 1, 0, 0, 0, true),
        (date_time_creation_for_invalid_date, 2022, 11, 31, 0, 0, 0, false),
        (date_time_creation_for_invalid_time, 2022, 1, 1, 0, 61, 0, false),
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn error_in_date_time_creation() {
        let result = create_date_time(2022, 11, 31, 0, 0, 0);
        assert_eq!(
            result,
            Err(GribError::InvalidValueError(
                "invalid date time: 2022-11-31 00:00:00".to_owned()
            ))
        );
    }

    #[test]
    fn test_utc_date_time_string() {
        let time = UtcDateTime::new(2025, 1, 1, 0, 0, 0);
        assert_eq!(format!("{time}"), "2025-01-01 00:00:00 UTC".to_owned())
    }
}
