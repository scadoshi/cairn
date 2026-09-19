//! What a counter and a day's entry are.

use chrono::NaiveDate;
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

/// A yearly target, at least 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goal(u32);

impl Goal {
    /// Rejects zero.
    pub fn per_year(n: u32) -> Result<Self, ValidationError> {
        if n == 0 {
            return Err(ValidationError::ZeroGoal);
        }
        Ok(Self(n))
    }

    /// The target for a full year.
    pub fn yearly(self) -> u32 {
        self.0
    }
}

/// A thing being counted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counter {
    /// Store id.
    pub id: CounterId,
    /// Display name.
    pub name: CounterName,
    /// Optional yearly target.
    pub goal: Option<Goal>,
    /// Day the counter was created; the odometer starts here.
    pub created_on: NaiveDate,
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
    fn goal_rejects_zero() {
        assert_eq!(Goal::per_year(0), Err(ValidationError::ZeroGoal));
        assert_eq!(Goal::per_year(5000).unwrap().yearly(), 5000);
    }
}
