//! The preferences that change what the numbers mean, kept together so the
//! store saves one row and the stats take one value.

use chrono::{Datelike, NaiveDate, NaiveDateTime, Weekday};
use serde::{Deserialize, Serialize};

/// How the counter list is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CounterOrder {
    /// Oldest first, the order they were made.
    #[default]
    Created,
    /// Alphabetical.
    Name,
    /// Highest this-year total first.
    MostActive,
    /// Highest lifetime first.
    Lifetime,
}

impl CounterOrder {
    /// Every option, for cycling.
    pub const ALL: [Self; 4] = [Self::Created, Self::Name, Self::MostActive, Self::Lifetime];

    /// Short label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Created => "Created",
            Self::Name => "Name",
            Self::MostActive => "Most active",
            Self::Lifetime => "Lifetime",
        }
    }

    /// The next option round.
    #[must_use]
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|o| *o == self).unwrap_or(0);
        Self::ALL
            .get((i + 1) % Self::ALL.len())
            .copied()
            .unwrap_or_default()
    }
}

/// Which mark the home screen shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Logo {
    /// The app's own mark.
    #[default]
    Cairn,
    /// The owner's dev mark, kept because this started as a personal app
    /// and the wordmark is his name.
    Scadoshi,
}

impl Logo {
    /// Every option, for cycling.
    pub const ALL: [Self; 2] = [Self::Cairn, Self::Scadoshi];

    /// Short label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Cairn => "Cairn",
            Self::Scadoshi => "scadoshi",
        }
    }

    /// The next option round.
    #[must_use]
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|o| *o == self).unwrap_or(0);
        Self::ALL
            .get((i + 1) % Self::ALL.len())
            .copied()
            .unwrap_or_default()
    }
}

/// What happens when a counter's daily goal is met.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Celebration {
    /// A sheen running up the screen.
    #[default]
    Sheen,
    /// Confetti in the theme's colors.
    Confetti,
    /// Nothing.
    Off,
}

impl Celebration {
    /// Every option, for cycling.
    pub const ALL: [Self; 3] = [Self::Sheen, Self::Confetti, Self::Off];

    /// Short label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Sheen => "Sheen",
            Self::Confetti => "Confetti",
            Self::Off => "Off",
        }
    }

    /// The next option round.
    #[must_use]
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|o| *o == self).unwrap_or(0);
        Self::ALL
            .get((i + 1) % Self::ALL.len())
            .copied()
            .unwrap_or_default()
    }
}

/// Lines shown when a goal is met, one picked per crossing.
///
/// Stepped through with a stride coprime with the length, the way the quotes
/// rotate, so the same one does not come up twice running and nothing has to
/// be stored to remember which was last.
const DONE_LINES: [&str; 11] = [
    "Day done",
    "Goal met",
    "That's the day",
    "Signed off",
    "Another stone on the pile",
    "Logged and done",
    "Today is paid",
    "Nothing owed",
    "Squared away",
    "That's the work",
    "Done and dusted",
];

/// Stride through [`DONE_LINES`]; 4 is coprime with 11.
const DONE_STRIDE: usize = 4;

/// A success line, varied by `nth` so consecutive crossings differ.
pub fn done_line(nth: u64) -> &'static str {
    let len = DONE_LINES.len();
    let i = usize::try_from(nth % len as u64).unwrap_or(0);
    DONE_LINES
        .get(i.wrapping_mul(DONE_STRIDE) % len)
        .copied()
        .unwrap_or("Goal met")
}

/// The settings that shape the statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    /// Monday or Sunday.
    pub week_start: Weekday,
    /// Hour of the day (0 to 6) at which a new day begins. A tap at 00:30
    /// with a rollover of 4 belongs to the day before.
    pub rollover_hour: u32,
    /// Weekdays that don't count against consistency or break a streak,
    /// as a bitmask with Monday at bit 0.
    pub rest_days: u8,
    /// Hourly chart divides by every day, not just active ones.
    pub hourly_all_days: bool,
    /// Counter list order.
    pub counter_order: CounterOrder,
    /// Minus buttons ask before subtracting.
    pub confirm_minus: bool,
    /// Which mark the home screen shows.
    pub logo: Logo,
    /// What happens when a daily goal is met.
    pub celebration: Celebration,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            week_start: Weekday::Mon,
            rollover_hour: 0,
            rest_days: 0,
            hourly_all_days: false,
            counter_order: CounterOrder::Created,
            confirm_minus: false,
            logo: Logo::Cairn,
            celebration: Celebration::Sheen,
        }
    }
}

impl Preferences {
    /// Rollover hours offered, on the hour, midnight to six.
    pub const ROLLOVER_HOURS: [u32; 4] = [0, 2, 4, 6];

    /// The calendar day a moment belongs to, given the rollover hour.
    pub fn day_of(&self, at: NaiveDateTime) -> NaiveDate {
        (at - chrono::Duration::hours(i64::from(self.rollover_hour))).date()
    }

    /// Whether a weekday is a rest day.
    pub fn is_rest(&self, day: NaiveDate) -> bool {
        self.rest_days & (1 << day.weekday().num_days_from_monday()) != 0
    }

    /// Flips a weekday's rest status.
    #[must_use]
    pub fn toggle_rest(self, weekday: Weekday) -> Self {
        Self {
            rest_days: self.rest_days ^ (1 << weekday.num_days_from_monday()),
            ..self
        }
    }

    /// The first day of the week containing `day`.
    pub fn week_start_of(&self, day: NaiveDate) -> NaiveDate {
        let offset =
            (7 + day.weekday().num_days_from_monday() - self.week_start.num_days_from_monday()) % 7;
        day - chrono::Duration::days(i64::from(offset))
    }

    /// Week number within the year: ISO weeks for a Monday start, the
    /// Sunday-based count (first Sunday starts week 1) otherwise.
    pub fn week_number(&self, day: NaiveDate) -> u32 {
        match self.week_start {
            Weekday::Sun => day.format("%U").to_string().parse::<u32>().unwrap_or(0) + 1,
            _ => day.iso_week().week(),
        }
    }

    /// Weekday names in this week's order, short.
    pub fn weekday_labels(&self) -> [&'static str; 7] {
        const MON: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let start = self.week_start.num_days_from_monday() as usize;
        let mut out = [""; 7];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = MON.get((start + i) % 7).copied().unwrap_or("");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn preferences_stored_before_a_field_existed_still_load() {
        // What a phone that last wrote settings before the logo option would
        // have in its settings row. Every field added later must fall back to
        // its default rather than failing the whole parse.
        let older = r#"{"week_start":"Mon","rollover_hour":4,"rest_days":6,
            "hourly_all_days":true,"counter_order":"Lifetime","confirm_minus":true}"#;
        let p: Preferences = serde_json::from_str(older).expect("old settings must still parse");
        assert_eq!(p.rollover_hour, 4);
        assert_eq!(p.rest_days, 6);
        assert_eq!(p.counter_order, CounterOrder::Lifetime);
        assert!(p.confirm_minus);
        assert_eq!(p.logo, Logo::Cairn, "a missing field takes its default");
    }

    #[test]
    fn every_success_line_is_reachable_before_any_repeats() {
        let mut seen = std::collections::HashSet::new();
        let mut last = "";
        for n in 0..DONE_LINES.len() as u64 {
            let line = done_line(n);
            assert_ne!(line, last, "the same line twice running at {n}");
            seen.insert(line);
            last = line;
        }
        assert_eq!(seen.len(), DONE_LINES.len(), "some line never shows");
    }

    #[test]
    fn the_celebration_option_cycles_through_every_choice() {
        let mut seen = Vec::new();
        let mut c = Celebration::default();
        for _ in 0..Celebration::ALL.len() {
            seen.push(c);
            c = c.next();
        }
        assert_eq!(c, Celebration::default());
        for option in Celebration::ALL {
            assert!(seen.contains(&option), "{option:?} is never reachable");
        }
    }

    #[test]
    fn the_logo_option_cycles_through_every_choice() {
        let mut seen = Vec::new();
        let mut l = Logo::default();
        for _ in 0..Logo::ALL.len() {
            seen.push(l);
            l = l.next();
        }
        assert_eq!(
            l,
            Logo::default(),
            "cycling must return to where it started"
        );
        for option in Logo::ALL {
            assert!(seen.contains(&option), "{option:?} is never reachable");
        }
    }

    #[test]
    fn rollover_moves_early_taps_to_the_day_before() {
        let p = Preferences {
            rollover_hour: 4,
            ..Default::default()
        };
        let late = d(2026, 9, 20).and_hms_opt(0, 30, 0).unwrap();
        assert_eq!(p.day_of(late), d(2026, 9, 19));
        let morning = d(2026, 9, 20).and_hms_opt(4, 0, 0).unwrap();
        assert_eq!(p.day_of(morning), d(2026, 9, 20));
        assert_eq!(Preferences::default().day_of(late), d(2026, 9, 20));
    }

    #[test]
    fn week_start_and_number_follow_the_setting() {
        // 2026-09-20 is a Sunday.
        let mon = Preferences::default();
        assert_eq!(mon.week_start_of(d(2026, 9, 20)), d(2026, 9, 14));
        assert_eq!(mon.week_number(d(2026, 9, 20)), 38);
        let sun = Preferences {
            week_start: Weekday::Sun,
            ..Default::default()
        };
        assert_eq!(sun.week_start_of(d(2026, 9, 20)), d(2026, 9, 20));
        assert_eq!(sun.week_start_of(d(2026, 9, 23)), d(2026, 9, 20));
        assert_eq!(sun.weekday_labels()[0], "Sun");
        assert_eq!(sun.week_number(d(2026, 9, 20)), 39);
    }

    #[test]
    fn rest_days_toggle_by_bit() {
        let p = Preferences::default().toggle_rest(Weekday::Sun);
        assert!(p.is_rest(d(2026, 9, 20)));
        assert!(!p.is_rest(d(2026, 9, 21)));
        assert!(!p.toggle_rest(Weekday::Sun).is_rest(d(2026, 9, 20)));
    }

    #[test]
    fn order_cycles() {
        assert_eq!(CounterOrder::Lifetime.next(), CounterOrder::Created);
        assert_eq!(CounterOrder::Created.next(), CounterOrder::Name);
    }
}
