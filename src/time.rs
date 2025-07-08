use std::fmt;

#[cfg(feature = "time-calculation")]
use chrono::{DateTime, LocalResult, TimeDelta, TimeZone, Utc};

#[cfg(feature = "time-calculation")]
use crate::codetables::grib2::Table4_4;
use crate::{codetables::grib2::Table1_2, Code, ForecastTime};

#[derive(Debug, PartialEq, Eq)]
/// Time-related raw information.
pub struct TemporalRawInfo {
    /// "Significance of reference time" set in Section 1 of the submessage. See
    /// [Code Table 1.2](crate::codetables::grib2::Table1_2).
    pub ref_time_significance: Code<Table1_2, u8>,
    /// "Reference time" set in Section 1 of the submessage.
    pub ref_time_unchecked: UtcDateTime,
    /// "Forecast time" set in Section 3 of the submessage.
    pub forecast_time_diff: Option<ForecastTime>,
}

impl TemporalRawInfo {
    pub(crate) fn new(
        ref_time_significance: u8,
        ref_time_unchecked: UtcDateTime,
        forecast_time_diff: Option<ForecastTime>,
    ) -> Self {
        let ref_time_significance = Table1_2::try_from(ref_time_significance).into();
        Self {
            ref_time_significance,
            ref_time_unchecked,
            forecast_time_diff,
        }
    }
}

#[cfg(feature = "time-calculation")]
#[derive(Debug, PartialEq, Eq)]
/// Time-related calculated information.
pub struct TemporalInfo {
    /// "Reference time" represented as [`chrono::DateTime`].
    pub ref_time: Option<DateTime<Utc>>,
    /// "Forecast time" calculated and represented as [`chrono::DateTime`].
    pub forecast_time_target: Option<DateTime<Utc>>,
}

#[cfg(feature = "time-calculation")]
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
/// UTC date and time container.
pub struct UtcDateTime {
    /// Year.
    pub year: u16,
    /// Month.
    pub month: u8,
    /// Day.
    pub day: u8,
    /// Hour.
    pub hour: u8,
    /// Minute.
    pub minute: u8,
    /// Second.
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

#[cfg(feature = "time-calculation")]
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

#[cfg(feature = "time-calculation")]
struct BasicTimeDelta(fn(i64) -> Option<TimeDelta>, i64);

#[cfg(feature = "time-calculation")]
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
            #[cfg(feature = "time-calculation")]
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
