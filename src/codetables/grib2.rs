use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Table1_2 {
    Analysis = 0,
    StartOfForecast,
    VerifyingTimeOfForecast,
    ObservationTime,
    LocalTime,
    SimulationStart,
    Missing = 255,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Table4_4 {
    Minute = 0,
    Hour,
    Day,
    Month,
    Year,
    Decade,
    Normal,
    Century,
    ThreeHours = 10,
    SixHours,
    TwelveHours,
    Second,
    Missing = 255,
}

impl Table4_4 {
    pub fn short_expr(&self) -> Option<&'static str> {
        match self {
            Self::Minute => Some("m"),
            Self::Hour => Some("h"),
            Self::Day => Some("D"),
            Self::Month => Some("M"),
            Self::Year => Some("Y"),
            Self::Decade => Some("10Y"),
            Self::Normal => Some("30Y"),
            Self::Century => Some("C"),
            Self::ThreeHours => Some("3h"),
            Self::SixHours => Some("6h"),
            Self::TwelveHours => Some("12h"),
            Self::Second => Some("s"),
            Self::Missing => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Table5_6 {
    FirstOrderSpatialDifferencing = 1,
    SecondOrderSpatialDifferencing,
    Missing = 255,
}

#[cfg(test)]
mod tests {
    use num_enum::TryFromPrimitiveError;

    use super::*;
    use crate::codetables::*;

    #[test]
    fn num_enum_conversion() {
        assert_eq!((Table4_4::try_from(1u8)), Ok(Table4_4::Hour));
        assert_eq!((Table4_4::try_from(10u8)), Ok(Table4_4::ThreeHours));
        assert_eq!(
            (Table4_4::try_from(254u8)),
            Err(TryFromPrimitiveError { number: 254 })
        );
    }

    #[test]
    fn num_lookup_result_conversion() {
        assert_eq!(Code::from(Table4_4::try_from(1u8)), Name(Table4_4::Hour));
        assert_eq!(Code::from(Table4_4::try_from(254u8)), Num(254));
    }
}
