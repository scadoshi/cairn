//! What a counter and a day's entry are.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Why a value could not be constructed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// Counter names must have at least one non-whitespace character.
    #[error("counter name cannot be empty")]
    EmptyName,
    /// Counter names are capped so they fit a nav row and a CSV filename.
    #[error("counter name cannot be longer than {max} characters")]
    NameTooLong {
        /// The cap that was exceeded.
        max: usize,
    },
    /// A goal of zero means "no goal"; use `None` for that instead.
    #[error("goal must be at least 1")]
    ZeroGoal,
    /// Steps come from a fixed menu so the buttons stay readable.
    #[error("step must be one of 1, 5, 10, 25, 50, 100")]
    BadStep,
}

/// Row id of a counter in the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CounterId(pub i64);

impl std::fmt::Display for CounterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A trimmed, non-empty counter name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CounterName(String);

impl CounterName {
    /// Longest name accepted, in characters.
    pub const MAX_LEN: usize = 48;

    /// Trims and validates.
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyName);
        }
        if trimmed.chars().count() > Self::MAX_LEN {
            return Err(ValidationError::NameTooLong { max: Self::MAX_LEN });
        }
        Ok(Self(trimmed.to_string()))
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Lowercase, dashes for anything that is not alphanumeric. For filenames.
    pub fn slug(&self) -> String {
        let mut out = String::with_capacity(self.0.len());
        let mut last_dash = true;
        for c in self.0.chars() {
            if c.is_alphanumeric() {
                out.extend(c.to_lowercase());
                last_dash = false;
            } else if !last_dash {
                out.push('-');
                last_dash = true;
            }
        }
        out.trim_end_matches('-').to_string()
    }
}

impl std::fmt::Display for CounterName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A target, at least 1, expressed the way the person thinks about it:
/// "5000 this year" or "15 a day". Pace math converts either to a yearly
/// figure for the year in question, so a per-day goal is exact on leap years.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Goal {
    /// Total for the calendar year.
    PerYear(u32),
    /// Reps every week; the yearly total is this times the weeks in the year.
    PerWeek(u32),
    /// Reps every day; the yearly total is this times the days in the year.
    PerDay(u32),
}

impl Goal {
    /// Rejects zero.
    pub fn per_year(n: u32) -> Result<Self, ValidationError> {
        if n == 0 {
            return Err(ValidationError::ZeroGoal);
        }
        Ok(Self::PerYear(n))
    }

    /// Rejects zero.
    pub fn per_week(n: u32) -> Result<Self, ValidationError> {
        if n == 0 {
            return Err(ValidationError::ZeroGoal);
        }
        Ok(Self::PerWeek(n))
    }

    /// Rejects zero.
    pub fn per_day(n: u32) -> Result<Self, ValidationError> {
        if n == 0 {
            return Err(ValidationError::ZeroGoal);
        }
        Ok(Self::PerDay(n))
    }

    /// The target for a full year of `days_in_year` days.
    pub fn yearly(self, days_in_year: u32) -> u32 {
        match self {
            Self::PerYear(n) => n,
            // 365 days is 52.14 weeks; round rather than floor so 100 a week
            // reads as 5,214, not 5,200.
            Self::PerWeek(n) => {
                let total = f64::from(n) * f64::from(days_in_year) / 7.0;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let rounded = total.round() as u32;
                rounded
            }
            Self::PerDay(n) => n.saturating_mul(days_in_year),
        }
    }

    /// The daily rate the goal implies in a year of `days_in_year` days.
    pub fn daily(self, days_in_year: u32) -> f64 {
        match self {
            Self::PerYear(n) => f64::from(n) / f64::from(days_in_year.max(1)),
            Self::PerWeek(n) => f64::from(n) / 7.0,
            Self::PerDay(n) => f64::from(n),
        }
    }

    /// Both figures as separate strings, the one entered first: `15/day` then
    /// `5,475/year`, or `10,000/year` then `27.4/day`.
    pub fn parts(self, days_in_year: u32) -> [String; 2] {
        use super::format::{rate, thousands};
        match self {
            Self::PerYear(n) => [
                format!("{}/year", thousands(n)),
                format!("{}/day", rate(self.daily(days_in_year))),
            ],
            Self::PerWeek(n) => [
                format!("{}/week", thousands(n)),
                format!("{}/year", thousands(self.yearly(days_in_year))),
            ],
            Self::PerDay(n) => [
                format!("{}/day", thousands(n)),
                format!("{}/year", thousands(self.yearly(days_in_year))),
            ],
        }
    }

    /// Both figures in one string, for prose: "15/day, 5,475/year".
    pub fn label(self, days_in_year: u32) -> String {
        self.parts(days_in_year).join(", ")
    }
}

impl std::fmt::Display for Goal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PerYear(n) => write!(f, "{n}/year"),
            Self::PerWeek(n) => write!(f, "{n}/week"),
            Self::PerDay(n) => write!(f, "{n}/day"),
        }
    }
}

/// How much one tap adds, from a fixed menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step(u32);

impl Step {
    /// The menu.
    pub const ALLOWED: [u32; 6] = [1, 5, 10, 25, 50, 100];

    /// Rejects anything not on the menu.
    pub fn new(n: u32) -> Result<Self, ValidationError> {
        if Self::ALLOWED.contains(&n) {
            Ok(Self(n))
        } else {
            Err(ValidationError::BadStep)
        }
    }

    /// The amount.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl Default for Step {
    fn default() -> Self {
        Self(1)
    }
}

/// A thing being counted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counter {
    /// Store id.
    pub id: CounterId,
    /// Display name.
    pub name: CounterName,
    /// Optional target.
    pub goal: Option<Goal>,
    /// How much one tap adds.
    pub step: Step,
    /// Day the counter was created; the odometer starts here.
    pub created_on: NaiveDate,
}

/// One tap: when it happened and by how much the count moved. Entries are
/// the per-day rollup; events are what hour-of-day statistics read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Local wall-clock time of the tap.
    pub at: NaiveDateTime,
    /// Signed change; -1 for an undo.
    pub delta: i64,
}

/// One counter's total for one day. Also the CSV row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayCount {
    /// Calendar day, local time.
    pub day: NaiveDate,
    /// Total logged that day.
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_trims_and_rejects_empty() {
        assert_eq!(
            CounterName::new("  pull-ups ").unwrap().as_str(),
            "pull-ups"
        );
        assert_eq!(CounterName::new("   "), Err(ValidationError::EmptyName));
    }

    #[test]
    fn name_caps_length() {
        let long = "x".repeat(CounterName::MAX_LEN + 1);
        assert!(matches!(
            CounterName::new(&long),
            Err(ValidationError::NameTooLong { .. })
        ));
    }

    #[test]
    fn slug_is_filename_safe() {
        assert_eq!(CounterName::new("Pull Ups!").unwrap().slug(), "pull-ups");
        assert_eq!(CounterName::new("  a  b ").unwrap().slug(), "a-b");
    }

    #[test]
    fn step_comes_from_the_menu() {
        assert_eq!(Step::new(25).unwrap().get(), 25);
        assert_eq!(Step::new(3), Err(ValidationError::BadStep));
        assert_eq!(Step::default().get(), 1);
    }

    #[test]
    fn goal_rejects_zero() {
        assert_eq!(Goal::per_year(0), Err(ValidationError::ZeroGoal));
        assert_eq!(Goal::per_day(0), Err(ValidationError::ZeroGoal));
    }

    #[test]
    fn goal_yearly_scales_per_day_by_the_year_length() {
        assert_eq!(Goal::per_year(5000).unwrap().yearly(365), 5000);
        assert_eq!(Goal::per_day(10).unwrap().yearly(365), 3650);
        assert_eq!(Goal::per_day(10).unwrap().yearly(366), 3660);
        assert_eq!(Goal::per_week(100).unwrap().yearly(365), 5214);
        assert_eq!(Goal::per_week(0), Err(ValidationError::ZeroGoal));
        assert_eq!(
            Goal::per_week(100).unwrap().label(365),
            "100/week, 5,214/year"
        );
        assert_eq!(Goal::per_day(15).unwrap().to_string(), "15/day");
        assert_eq!(Goal::per_year(5000).unwrap().to_string(), "5000/year");
        assert_eq!(Goal::per_day(15).unwrap().label(365), "15/day, 5,475/year");
        assert_eq!(
            Goal::per_year(3650).unwrap().label(365),
            "3,650/year, 10.0/day"
        );
    }
}
