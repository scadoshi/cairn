//! How dates are written, a preference: month-first by default, day-first
//! for everyone outside the US.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// The two orders. Both use two-digit years and slashes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DateFormat {
    /// 09/20/26
    #[default]
    MonthDayYear,
    /// 20/09/26
    DayMonthYear,
}

impl DateFormat {
    /// Every option, for a picker.
    pub const ALL: [Self; 2] = [Self::MonthDayYear, Self::DayMonthYear];

    /// The pattern as a person would write it.
    pub fn label(self) -> &'static str {
        match self {
            Self::MonthDayYear => "MM/DD/YY",
            Self::DayMonthYear => "DD/MM/YY",
        }
    }

    /// Stable key for storage.
    pub fn key(self) -> &'static str {
        match self {
            Self::MonthDayYear => "mdy",
            Self::DayMonthYear => "dmy",
        }
    }

    /// Parses a stored key; anything unknown is the default.
    pub fn from_key(key: &str) -> Self {
        match key {
            "dmy" => Self::DayMonthYear,
            _ => Self::MonthDayYear,
        }
    }

    /// The other one, for a toggle.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::MonthDayYear => Self::DayMonthYear,
            Self::DayMonthYear => Self::MonthDayYear,
        }
    }

    /// Full date: 09/20/26 or 20/09/26.
    pub fn date(self, d: NaiveDate) -> String {
        match self {
            Self::MonthDayYear => d.format("%m/%d/%y").to_string(),
            Self::DayMonthYear => d.format("%d/%m/%y").to_string(),
        }
    }

    /// Without the year, for chart ticks and week starts: 09/20 or 20/09.
    pub fn short(self, d: NaiveDate) -> String {
        match self {
            Self::MonthDayYear => d.format("%m/%d").to_string(),
            Self::DayMonthYear => d.format("%d/%m").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_orders_and_round_trip() {
        let d = NaiveDate::from_ymd_opt(2026, 9, 20).unwrap();
        assert_eq!(DateFormat::MonthDayYear.date(d), "09/20/26");
        assert_eq!(DateFormat::DayMonthYear.date(d), "20/09/26");
        assert_eq!(DateFormat::DayMonthYear.short(d), "20/09");
        assert_eq!(
            DateFormat::from_key(DateFormat::DayMonthYear.key()),
            DateFormat::DayMonthYear
        );
        assert_eq!(DateFormat::from_key("garbage"), DateFormat::MonthDayYear);
        assert_eq!(DateFormat::MonthDayYear.next(), DateFormat::DayMonthYear);
    }
}
