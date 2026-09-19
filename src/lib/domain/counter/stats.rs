//! Everything derived from a counter's entries: the odometer reading, this
//! year's numbers, and standing against a yearly goal.
//!
//! All functions take `today` as a parameter. The clock is read once, at the
//! UI edge, so these are testable with fixed dates.

use super::{DayCount, Goal};
use chrono::{Datelike, NaiveDate};

/// Standing against a yearly goal as of `today`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pace {
    /// The full-year target.
    pub goal: u32,
    /// Where the total should be today to finish exactly on time.
    pub target_today: u32,
    /// Total this year minus the target. Negative means behind.
    pub delta: i64,
    /// Per-day rate over the remaining days (today included) to still make it.
    /// `None` when the goal is already met.
    pub needed_per_day: Option<f64>,
    /// Remaining count, zero once met.
    pub remaining: u32,
}

/// One calendar year of a counter.
#[derive(Debug, Clone, PartialEq)]
pub struct YearSummary {
    /// The year.
    pub year: i32,
    /// Total logged that year.
    pub total: u32,
    /// Days elapsed in the year through `today` (or the full year if past).
    pub days_elapsed: u32,
    /// Days in the year: 365 or 366.
    pub days_in_year: u32,
    /// `total / days_elapsed`.
    pub per_day: f64,
    /// Number of days with at least one count.
    pub active_days: u32,
    /// Standing against the goal, if one is set and this is the current year.
    pub pace: Option<Pace>,
}

/// The whole picture for one counter as of `today`.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    /// Today's count, zero if no row.
    pub today: u32,
    /// The odometer: every entry ever.
    pub lifetime: u32,
    /// Per-day average since the first entry (inclusive) through today.
    pub lifetime_per_day: f64,
    /// Total for the calendar month containing `today`.
    pub this_month: u32,
    /// The current year.
    pub this_year: YearSummary,
    /// Every year with at least one entry, newest first. Includes the current
    /// year even when it has none.
    pub years: Vec<YearSummary>,
    /// The single biggest day, if any.
    pub best_day: Option<DayCount>,
    /// Consecutive days ending today (or yesterday, if today is still empty)
    /// with a non-zero count.
    pub streak: u32,
}

/// 365 or 366.
pub fn days_in_year(year: i32) -> u32 {
    if NaiveDate::from_ymd_opt(year, 1, 1).is_some_and(|d| d.leap_year()) {
        366
    } else {
        365
    }
}

#[allow(clippy::cast_precision_loss)]
fn ratio(total: u32, days: u32) -> f64 {
    if days == 0 {
        0.0
    } else {
        f64::from(total) / f64::from(days)
    }
}

/// Standing against `goal` given `total_this_year` on `today`.
pub fn pace(goal: Goal, total_this_year: u32, today: NaiveDate) -> Pace {
    let year_len = days_in_year(today.year());
    let day = today.ordinal();
    let yearly = goal.yearly(year_len);
    // yearly * day / year_len, in u64 so a big goal can't overflow u32.
    let target_today = (u64::from(yearly) * u64::from(day) / u64::from(year_len))
        .try_into()
        .unwrap_or(u32::MAX);
    let remaining = yearly.saturating_sub(total_this_year);
    let days_left = year_len.saturating_sub(day).saturating_add(1);
    let needed_per_day = (remaining > 0).then(|| ratio(remaining, days_left));
    Pace {
        goal: yearly,
        target_today,
        delta: i64::from(total_this_year) - i64::from(target_today),
        needed_per_day,
        remaining,
    }
}

fn summarize_year(
    entries: &[DayCount],
    year: i32,
    today: NaiveDate,
    goal: Option<Goal>,
) -> YearSummary {
    let in_year = entries.iter().filter(|e| e.day.year() == year);
    let mut total = 0u32;
    let mut active_days = 0u32;
    for e in in_year {
        total = total.saturating_add(e.count);
        if e.count > 0 {
            active_days += 1;
        }
    }
    let days_in_year = days_in_year(year);
    let days_elapsed = match year.cmp(&today.year()) {
        std::cmp::Ordering::Less => days_in_year,
        std::cmp::Ordering::Greater => 0,
        std::cmp::Ordering::Equal => today.ordinal(),
    };
    let pace = match goal {
        Some(g) if year == today.year() => Some(pace(g, total, today)),
        _ => None,
    };
    YearSummary {
        year,
        total,
        days_elapsed,
        days_in_year,
        per_day: ratio(total, days_elapsed),
        active_days,
        pace,
    }
}

fn streak(entries: &[DayCount], today: NaiveDate) -> u32 {
    let has = |d: NaiveDate| entries.iter().any(|e| e.day == d && e.count > 0);
    let mut cursor = if has(today) {
        today
    } else {
        match today.pred_opt() {
            Some(y) if has(y) => y,
            _ => return 0,
        }
    };
    let mut n = 0u32;
    while has(cursor) {
        n += 1;
        match cursor.pred_opt() {
            Some(prev) => cursor = prev,
            None => break,
        }
    }
    n
}

/// Computes the full [`Summary`] for one counter.
pub fn summarize(entries: &[DayCount], goal: Option<Goal>, today: NaiveDate) -> Summary {
    let today_count = entries
        .iter()
        .find(|e| e.day == today)
        .map_or(0, |e| e.count);
    let lifetime = entries
        .iter()
        .fold(0u32, |acc, e| acc.saturating_add(e.count));
    let first_day = entries.iter().map(|e| e.day).min();
    let lifetime_days = first_day
        .and_then(|first| u32::try_from((today - first).num_days() + 1).ok())
        .unwrap_or(0);
    let this_month = entries
        .iter()
        .filter(|e| e.day.year() == today.year() && e.day.month() == today.month())
        .fold(0u32, |acc, e| acc.saturating_add(e.count));

    let mut year_list: Vec<i32> = entries.iter().map(|e| e.day.year()).collect();
    year_list.push(today.year());
    year_list.sort_unstable();
    year_list.dedup();
    year_list.reverse();
    let years: Vec<YearSummary> = year_list
        .into_iter()
        .map(|y| summarize_year(entries, y, today, goal))
        .collect();
    let this_year = summarize_year(entries, today.year(), today, goal);

    let best_day = entries
        .iter()
        .filter(|e| e.count > 0)
        .max_by_key(|e| (e.count, std::cmp::Reverse(e.day)))
        .copied();

    Summary {
        today: today_count,
        lifetime,
        lifetime_per_day: ratio(lifetime, lifetime_days),
        this_month,
        this_year,
        years,
        best_day,
        streak: streak(entries, today),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn e(y: i32, m: u32, day: u32, count: u32) -> DayCount {
        DayCount {
            day: d(y, m, day),
            count,
        }
    }

    #[test]
    fn empty_counter_is_all_zero() {
        let s = summarize(&[], None, d(2026, 9, 19));
        assert_eq!(s.lifetime, 0);
        assert_eq!(s.today, 0);
        assert_eq!(s.streak, 0);
        assert_eq!(s.best_day, None);
        assert_eq!(s.years.len(), 1);
        assert_eq!(s.this_year.days_elapsed, 262);
        assert!(s.this_year.per_day.abs() < f64::EPSILON);
    }

    #[test]
    fn lifetime_spans_years_and_per_day_uses_first_entry() {
        let entries = [e(2025, 12, 31, 10), e(2026, 1, 1, 20), e(2026, 1, 2, 30)];
        let s = summarize(&entries, None, d(2026, 1, 2));
        assert_eq!(s.lifetime, 60);
        // 3 days inclusive
        assert!((s.lifetime_per_day - 20.0).abs() < f64::EPSILON);
        assert_eq!(s.this_year.total, 50);
        assert_eq!(s.this_month, 50);
        assert_eq!(
            s.years.iter().map(|y| y.year).collect::<Vec<_>>(),
            vec![2026, 2025]
        );
        assert_eq!(s.years[1].days_elapsed, 365);
        assert_eq!(s.today, 30);
    }

    #[test]
    fn year_per_day_divides_by_days_elapsed() {
        let entries = [e(2026, 1, 1, 100)];
        let s = summarize(&entries, None, d(2026, 1, 10));
        assert!((s.this_year.per_day - 10.0).abs() < f64::EPSILON);
        assert_eq!(s.this_year.active_days, 1);
    }

    #[test]
    fn pace_on_track_and_behind() {
        let goal = Goal::per_year(3650).unwrap();
        // Day 100 of a 365-day year: target is 1000.
        let p = pace(goal, 1000, d(2026, 4, 10));
        assert_eq!(d(2026, 4, 10).ordinal(), 100);
        assert_eq!(p.target_today, 1000);
        assert_eq!(p.delta, 0);
        assert_eq!(p.remaining, 2650);
        // 266 days left including today.
        assert!((p.needed_per_day.unwrap() - 2650.0 / 266.0).abs() < 1e-9);

        let behind = pace(goal, 900, d(2026, 4, 10));
        assert_eq!(behind.delta, -100);
    }

    #[test]
    fn per_day_goal_targets_exactly_day_times_rate() {
        let goal = Goal::per_day(10).unwrap();
        let p = pace(goal, 950, d(2026, 4, 10));
        assert_eq!(p.goal, 3650);
        assert_eq!(p.target_today, 1000);
        assert_eq!(p.delta, -50);
        // A per-day goal behind pace still needs more than the base rate.
        assert!(p.needed_per_day.unwrap() > 10.0);
    }

    #[test]
    fn pace_met_has_no_needed_rate() {
        let goal = Goal::per_year(10).unwrap();
        let p = pace(goal, 12, d(2026, 6, 1));
        assert_eq!(p.remaining, 0);
        assert_eq!(p.needed_per_day, None);
    }

    #[test]
    fn leap_year_uses_366() {
        let goal = Goal::per_year(366).unwrap();
        let p = pace(goal, 0, d(2028, 12, 31));
        assert_eq!(p.target_today, 366);
        assert_eq!(p.needed_per_day, Some(366.0));
    }

    #[test]
    fn pace_only_on_current_year() {
        let goal = Goal::per_year(100).unwrap();
        let s = summarize(&[e(2025, 5, 5, 1)], Some(goal), d(2026, 1, 1));
        assert!(s.this_year.pace.is_some());
        assert!(s.years[1].pace.is_none());
    }

    #[test]
    fn streak_counts_back_from_today_or_yesterday() {
        let entries = [e(2026, 9, 17, 1), e(2026, 9, 18, 1), e(2026, 9, 19, 1)];
        assert_eq!(streak(&entries, d(2026, 9, 19)), 3);
        // Today empty, yesterday counted: streak still alive.
        assert_eq!(streak(&entries, d(2026, 9, 20)), 3);
        // Two days gap: broken.
        assert_eq!(streak(&entries, d(2026, 9, 21)), 0);
        // Zero rows don't count.
        let with_zero = [e(2026, 9, 18, 0), e(2026, 9, 19, 5)];
        assert_eq!(streak(&with_zero, d(2026, 9, 19)), 1);
    }

    #[test]
    fn best_day_prefers_earliest_on_tie() {
        let entries = [e(2026, 1, 1, 50), e(2026, 1, 2, 50), e(2026, 1, 3, 10)];
        let s = summarize(&entries, None, d(2026, 1, 3));
        assert_eq!(s.best_day, Some(e(2026, 1, 1, 50)));
    }
}
