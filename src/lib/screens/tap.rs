//! A tap on a counter's step buttons, from Home's cards or the counter's own
//! screen: what it writes and what it says.
//!
//! One rule for both screens, so a crossing looks the same from either. A
//! tap is two steps around the store's write: [`Tap::plan`] says whether
//! there is anything to write, and [`Tap::outcome`] says what an applied tap
//! shows.

use chrono::{Datelike, NaiveDate};

use crate::domain::counter::models::{DayCount, Goal};
use crate::domain::counter::stats;
use crate::domain::preferences::{self, Celebration};
use crate::screens::saying::Saying;
use crate::screens::toast::Toast;

/// A tap on one counter, with the counter as the store has it at the tap.
#[derive(Debug, Clone, Copy)]
pub struct Tap<'entries> {
    /// The counter's name, for the "-0" toast.
    pub name: &'entries str,
    /// Every day the counter has logged, read from the store at the tap
    /// rather than from the last render: the day may have turned since.
    pub entries: &'entries [DayCount],
    /// The counter's goal, if it has one.
    pub goal: Option<Goal>,
    /// The counter's own celebration; `None` follows the app's.
    pub own: Option<Celebration>,
    /// The app's celebration setting.
    pub app: Celebration,
    /// The day the tap counts on.
    pub day: NaiveDate,
}

/// Whether a tap writes anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// Nothing to write: a minus on an empty day. Show the toast.
    Nothing(Toast),
    /// Write this delta, which a minus may have trimmed so the day stays at
    /// zero or above.
    Apply(i64),
}

/// What a tap the store applied shows.
#[derive(Debug, Clone)]
pub enum Outcome {
    /// Nothing: the count it moved is on the screen.
    Quiet,
    /// A celebration, which the client plays and toasts the line of.
    Celebrate {
        /// The animation, the counter's own choice or else the app's.
        how: Celebration,
        /// What it says; `None` is the usual rotating line.
        saying: Option<Saying>,
    },
}

impl Tap<'_> {
    /// The day's count before the tap.
    pub fn today(&self) -> u32 {
        self.entries
            .iter()
            .find(|e| e.day == self.day)
            .map_or(0, |e| e.count)
    }

    /// The year's total before the tap.
    pub fn year_before(&self) -> u32 {
        self.entries
            .iter()
            .filter(|e| e.day.year() == self.day.year())
            .fold(0, |total, e| total.saturating_add(e.count))
    }

    /// Whether the tap of `delta` writes anything.
    ///
    /// Reports what actually happens rather than what was asked for: a day
    /// cannot go below zero, so a minus on an empty day reads "-0" without
    /// a case of its own.
    pub fn plan(&self, delta: i64) -> Plan {
        match stats::applied_delta(self.today(), delta) {
            0 => Plan::Nothing(Toast::info(format!("{} -0", self.name))),
            applied => Plan::Apply(applied),
        }
    }

    /// What the tap shows once the store applied `applied`.
    ///
    /// `all_met` says whether every counter's goal for the day is met, the
    /// tapped one included. It is asked only when this tap finished its own
    /// day, since no other tap can have finished them all.
    ///
    /// Asked of the tap rather than of the screen, so nothing is stored to
    /// remember what was already said, and reopening the app on a finished
    /// day is silent.
    pub fn outcome(&self, applied: i64, all_met: impl FnOnce() -> bool) -> Outcome {
        let Some(goal) = self.goal else {
            return Outcome::Quiet;
        };
        let days = stats::days_in_year(self.day.year());
        let done = stats::crosses_goal(goal, self.today(), applied, days);
        let mark = stats::goal_mark_crossed(goal.yearly(days), self.year_before(), applied);
        if !done && mark.is_none() {
            return Outcome::Quiet;
        }
        let how = self.own.unwrap_or(self.app);
        if how == Celebration::Off {
            return Outcome::Quiet;
        }
        // Rarest first. A fraction of the year comes round a few times a
        // year, the whole day most days, one counter's day several times a
        // day.
        let saying = if let Some(pct) = mark {
            Some(Saying::Fixed(format!("{pct}% of the year")))
        } else if done && all_met() {
            Some(Saying::Rotating(preferences::all_done_line))
        } else {
            None
        };
        Outcome::Celebrate { how, saying }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, m, d).unwrap()
    }

    fn entry(m: u32, d: u32, count: u32) -> DayCount {
        DayCount {
            day: day(m, d),
            count,
        }
    }

    fn tap(entries: &[DayCount], goal: Option<Goal>) -> Tap<'_> {
        Tap {
            name: "Pushups",
            entries,
            goal,
            own: None,
            app: Celebration::Confetti,
            day: day(11, 8),
        }
    }

    fn never() -> bool {
        panic!("all_met is asked only when the tap finished its day")
    }

    #[test]
    fn a_minus_on_an_empty_day_writes_nothing_and_says_minus_zero() {
        let entries = [entry(11, 7, 30)];
        assert_eq!(
            tap(&entries, None).plan(-1),
            Plan::Nothing(Toast::info("Pushups -0"))
        );
    }

    #[test]
    fn a_minus_past_zero_is_trimmed_to_the_day() {
        let entries = [entry(11, 8, 3)];
        assert_eq!(tap(&entries, None).plan(-10), Plan::Apply(-3));
        assert_eq!(tap(&entries, None).plan(5), Plan::Apply(5));
    }

    #[test]
    fn today_and_the_year_come_from_the_entries() {
        let entries = [
            entry(11, 8, 4),
            entry(11, 1, 6),
            DayCount {
                day: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
                count: 100,
            },
        ];
        let t = tap(&entries, None);
        assert_eq!(t.today(), 4);
        assert_eq!(t.year_before(), 10, "last year's days are not this year's");
    }

    #[test]
    fn a_plain_plus_one_is_quiet() {
        let entries = [entry(11, 8, 3)];
        let goal = Some(Goal::PerDay(10));
        assert!(matches!(
            tap(&entries, goal).outcome(1, never),
            Outcome::Quiet
        ));
        assert!(matches!(
            tap(&entries, None).outcome(1, never),
            Outcome::Quiet
        ));
    }

    #[test]
    fn finishing_the_day_celebrates_with_the_usual_line() {
        let entries = [entry(11, 8, 9)];
        let out = tap(&entries, Some(Goal::PerDay(10))).outcome(1, || false);
        assert!(
            matches!(
                out,
                Outcome::Celebrate {
                    how: Celebration::Confetti,
                    saying: None
                }
            ),
            "{out:?}"
        );
    }

    #[test]
    fn finishing_every_goal_says_the_all_done_line() {
        let entries = [entry(11, 8, 9)];
        let out = tap(&entries, Some(Goal::PerDay(10))).outcome(1, || true);
        let Outcome::Celebrate {
            saying: Some(Saying::Rotating(pick)),
            ..
        } = out
        else {
            panic!("{out:?}");
        };
        assert_eq!(pick(0), preferences::all_done_line(0));
    }

    #[test]
    fn a_year_mark_wins_over_all_done() {
        // 1,000 a year: 99 before, so the tap of 1 reaches 10%.
        let entries = [entry(11, 1, 99)];
        let out = tap(&entries, Some(Goal::PerYear(1000))).outcome(1, || true);
        let Outcome::Celebrate {
            saying: Some(Saying::Fixed(text)),
            ..
        } = out
        else {
            panic!("{out:?}");
        };
        assert_eq!(text, "10% of the year");
    }

    #[test]
    fn a_year_mark_celebrates_without_finishing_the_day() {
        let entries = [entry(11, 1, 99)];
        let out = tap(&entries, Some(Goal::PerYear(1_000_000))).outcome(1, never);
        assert!(matches!(out, Outcome::Quiet), "{out:?}");
        let out = tap(&entries, Some(Goal::PerYear(1000))).outcome(1, || false);
        assert!(
            matches!(
                out,
                Outcome::Celebrate {
                    saying: Some(Saying::Fixed(_)),
                    ..
                }
            ),
            "{out:?}"
        );
    }

    #[test]
    fn the_counters_own_celebration_wins_over_the_apps() {
        let entries = [entry(11, 8, 9)];
        let mut t = tap(&entries, Some(Goal::PerDay(10)));
        t.own = Some(Celebration::Poppers);
        let out = t.outcome(1, || false);
        assert!(
            matches!(
                out,
                Outcome::Celebrate {
                    how: Celebration::Poppers,
                    ..
                }
            ),
            "{out:?}"
        );
    }

    #[test]
    fn off_is_quiet() {
        let entries = [entry(11, 8, 9)];
        let mut t = tap(&entries, Some(Goal::PerDay(10)));
        t.app = Celebration::Off;
        assert!(matches!(t.outcome(1, || true), Outcome::Quiet));
        t.app = Celebration::Confetti;
        t.own = Some(Celebration::Off);
        assert!(matches!(t.outcome(1, || true), Outcome::Quiet));
    }

    #[test]
    fn a_minus_never_celebrates() {
        let entries = [entry(11, 8, 10)];
        let out = tap(&entries, Some(Goal::PerDay(10))).outcome(-1, never);
        assert!(matches!(out, Outcome::Quiet));
    }
}
