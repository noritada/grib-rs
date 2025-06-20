use std::fmt;

#[cfg(feature = "chrono")]
use chrono::{DateTime, LocalResult, TimeZone, Utc};

use crate::ForecastTime;

pub struct TemporalInfo {
    pub ref_time_significance: u8,
    pub ref_time_unchecked: UtcDateTime,
    #[cfg(feature = "chrono")]
    pub ref_time: Option<DateTime<Utc>>,
    pub forecast_time: Option<ForecastTime>,
}

impl TemporalInfo {
    pub(crate) fn new(
        ref_time_significance: u8,
        ref_time_unchecked: UtcDateTime,
        forecast_time: Option<ForecastTime>,
    ) -> Self {
        #[cfg(feature = "chrono")]
        let ref_time = create_date_time(
            ref_time_unchecked.year.into(),
            ref_time_unchecked.month.into(),
            ref_time_unchecked.day.into(),
            ref_time_unchecked.hour.into(),
            ref_time_unchecked.minute.into(),
            ref_time_unchecked.second.into(),
        );
        Self {
            ref_time_significance,
            ref_time_unchecked,
            #[cfg(feature = "chrono")]
            ref_time,
            forecast_time,
        }
    }
}

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
fn create_date_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<DateTime<Utc>> {
    let result = Utc.with_ymd_and_hms(year, month, day, hour, minute, second);
    if let LocalResult::None = result {
        None
    } else {
        Some(result.unwrap())
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
                assert_eq!(result.is_some(), $ok_expected);
            }
        )*);
    }

    test_date_time_creation! {
        (date_time_creation_for_valid_date_time, 2022, 1, 1, 0, 0, 0, true),
        (date_time_creation_for_invalid_date, 2022, 11, 31, 0, 0, 0, false),
        (date_time_creation_for_invalid_time, 2022, 1, 1, 0, 61, 0, false),
    }

    #[test]
    fn test_utc_date_time_string() {
        let time = UtcDateTime::new(2025, 1, 1, 0, 0, 0);
        assert_eq!(format!("{time}"), "2025-01-01 00:00:00 UTC".to_owned())
    }
}
