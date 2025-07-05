use std::fmt;

#[cfg(feature = "chrono")]
use chrono::{DateTime, LocalResult, TimeDelta, TimeZone, Utc};

use crate::ForecastTime;
#[cfg(feature = "chrono")]
use crate::{codetables::grib2::Table4_4, Code};

pub struct TemporalRawInfo {
    pub ref_time_significance: u8,
    pub ref_time_unchecked: UtcDateTime,
    pub forecast_time_diff: Option<ForecastTime>,
}

impl TemporalRawInfo {
    pub(crate) fn new(
        ref_time_significance: u8,
        ref_time_unchecked: UtcDateTime,
        forecast_time_diff: Option<ForecastTime>,
    ) -> Self {
        Self {
            ref_time_significance,
            ref_time_unchecked,
            forecast_time_diff,
        }
    }
}

#[cfg(feature = "chrono")]
pub struct TemporalInfo {
    pub ref_time: Option<DateTime<Utc>>,
    pub forecast_time_target: Option<DateTime<Utc>>,
}

#[cfg(feature = "chrono")]
impl From<&TemporalRawInfo> for TemporalInfo {
    fn from(value: &TemporalRawInfo) -> Self {
        let ref_time = create_date_time(
            value.ref_time_unchecked.year.into(),
            value.ref_time_unchecked.month.into(),
            value.ref_time_unchecked.day.into(),
            value.ref_time_unchecked.hour.into(),
            value.ref_time_unchecked.minute.into(),
            value.ref_time_unchecked.second.into(),
        );
        let forecast_time_delta = value
            .forecast_time_diff
            .as_ref()
            .and_then(|ft| BasicTimeDelta::new(ft).and_then(|td| td.convert(ft)));
        let forecast_time_target = ref_time
            .zip(forecast_time_delta)
            .map(|(time, delta)| time + delta);
        Self {
            ref_time,
            forecast_time_target,
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

#[cfg(feature = "chrono")]
struct BasicTimeDelta(fn(i64) -> Option<TimeDelta>, i64);

#[cfg(feature = "chrono")]
impl BasicTimeDelta {
    fn new(ft: &ForecastTime) -> Option<Self> {
        match ft.unit {
            Code::Name(Table4_4::Second) => Some(Self(TimeDelta::try_seconds, 1)),
            Code::Name(Table4_4::Minute) => Some(Self(TimeDelta::try_minutes, 1)),
            Code::Name(Table4_4::Hour) => Some(Self(TimeDelta::try_hours, 1)),
            Code::Name(Table4_4::ThreeHours) => Some(Self(TimeDelta::try_hours, 3)),
            Code::Name(Table4_4::SixHours) => Some(Self(TimeDelta::try_hours, 6)),
            Code::Name(Table4_4::TwelveHours) => Some(Self(TimeDelta::try_hours, 12)),
            Code::Name(Table4_4::Day) => Some(Self(TimeDelta::try_days, 1)),
            _ => None,
        }
    }

    fn convert(&self, ft: &ForecastTime) -> Option<TimeDelta> {
        let BasicTimeDelta(func, value) = self;
        func(i64::try_from(ft.value).ok()? * value)
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
