//! Everything derived from a counter's entries: the odometer reading, this
//! year's numbers, and standing against a yearly goal.
//!
//! All functions take `today` as a parameter. The clock is read once, at the
//! UI edge, so these are testable with fixed dates.

use super::{DayCount, Goal};
use crate::domain::preferences::Preferences;
use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap;

/// Standing against a yearly goal as of `today`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pace {
    /// The full-year target.
    pub goal: u32,
    /// Where the total should be today to finish exactly on time.
    pub target_today: u32,
    /// Total this year minus the target. Negative means behind.
    pub delta: i64,
    /// Per-day rate over the remaining days (today included) to land exactly
    /// on the goal. Negative once the goal is passed.
    pub needed_per_day: f64,
    /// Goal minus the total so far. Negative once the goal is passed.
    pub remaining: i64,
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
    /// The longest run of consecutive logged days ever.
    pub longest_streak: u32,
    /// Days since the last logged day; zero if today is logged, `None` if
    /// nothing was ever logged.
    pub days_since_last: Option<u32>,
    /// Active days this year over days elapsed, as a fraction 0 to 1.
    pub consistency: f64,
    /// Year-end total if the rest of the year keeps this year's per-day rate.
    pub projected_year_end: u32,
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
    let remaining = i64::from(yearly) - i64::from(total_this_year);
    let days_left = year_len.saturating_sub(day).saturating_add(1).max(1);
    #[allow(clippy::cast_precision_loss)]
    let needed_per_day = remaining as f64 / f64::from(days_left);
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

/// Whether `prev` to `next` is one step of a streak: consecutive days, or
/// separated only by rest days, which neither extend nor break it.
fn adjacent(prev: NaiveDate, next: NaiveDate, prefs: &Preferences) -> bool {
    let mut d = prev;
    loop {
        let Some(n) = d.succ_opt() else {
            return false;
        };
        if n == next {
            return true;
        }
        if !prefs.is_rest(n) {
            return false;
        }
        d = n;
    }
}

fn longest_streak(entries: &[DayCount], prefs: &Preferences) -> u32 {
    let mut days: Vec<NaiveDate> = entries
        .iter()
        .filter(|e| e.count > 0)
        .map(|e| e.day)
        .collect();
    days.sort_unstable();
    days.dedup();
    let mut best = 0u32;
    let mut run = 0u32;
    let mut prev: Option<NaiveDate> = None;
    for d in days {
        run = match prev {
            Some(p) if adjacent(p, d, prefs) => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(d);
    }
    best
}

/// Consecutive logged days ending today (or yesterday if today is still
/// empty), stepping over rest days.
fn streak(entries: &[DayCount], today: NaiveDate, prefs: &Preferences) -> u32 {
    let has = |d: NaiveDate| entries.iter().any(|e| e.day == d && e.count > 0);
    // Walk back over today and any rest days to the last day that counts.
    let mut cursor = today;
    let mut allow_empty = true; // today may be unlogged without breaking
    loop {
        if has(cursor) {
            break;
        }
        if prefs.is_rest(cursor) || allow_empty {
            allow_empty = false;
            match cursor.pred_opt() {
                Some(p) => cursor = p,
                None => return 0,
            }
        } else {
            return 0;
        }
    }
    let mut n = 0u32;
    loop {
        if has(cursor) {
            n += 1;
        } else if !prefs.is_rest(cursor) {
            break;
        }
        match cursor.pred_opt() {
            Some(prev) => cursor = prev,
            None => break,
        }
    }
    n
}

/// Elapsed days of `year` through `today` that are not rest days.
fn counting_days(year: i32, today: NaiveDate, prefs: &Preferences) -> u32 {
    let Some(mut d) = NaiveDate::from_ymd_opt(year, 1, 1) else {
        return 0;
    };
    let end = if year < today.year() {
        NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(today)
    } else {
        today
    };
    let mut n = 0u32;
    while d <= end {
        if !prefs.is_rest(d) {
            n += 1;
        }
        match d.succ_opt() {
            Some(next) => d = next,
            None => break,
        }
    }
    n
}

/// Computes the full [`Summary`] for one counter with default rules.
pub fn summarize(entries: &[DayCount], goal: Option<Goal>, today: NaiveDate) -> Summary {
    summarize_with(entries, goal, today, &Preferences::default())
}

/// Computes the full [`Summary`] for one counter under the given rules:
/// rest days don't break streaks or count against consistency.
pub fn summarize_with(
    entries: &[DayCount],
    goal: Option<Goal>,
    today: NaiveDate,
    prefs: &Preferences,
) -> Summary {
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
    let last_logged = entries.iter().filter(|e| e.count > 0).map(|e| e.day).max();
    let days_since_last = last_logged.and_then(|d| u32::try_from((today - d).num_days()).ok());
    let consistency = ratio(
        this_year.active_days,
        counting_days(today.year(), today, prefs)
            .min(this_year.days_elapsed)
            .max(1),
    )
    .min(1.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let projected_year_end = (this_year.per_day * f64::from(this_year.days_in_year)).round() as u32;

    Summary {
        today: today_count,
        lifetime,
        lifetime_per_day: ratio(lifetime, lifetime_days),
        this_month,
        this_year,
        years,
        best_day,
        streak: streak(entries, today, prefs),
        longest_streak: longest_streak(entries, prefs),
        days_since_last,
        consistency,
        projected_year_end,
    }
}

/// One day's total across several counters, newest last.
///
/// The home screen's figures are about the day rather than any one counter,
/// so they run over this instead of over each counter in turn. Days that only
/// some counters logged still appear, carrying the counters that did.
///
/// Every list is assumed sorted by day, which is how the store returns them.
pub fn merge_days<'a>(per_counter: impl IntoIterator<Item = &'a [DayCount]>) -> Vec<DayCount> {
    let mut totals: BTreeMap<NaiveDate, u32> = BTreeMap::new();
    for entries in per_counter {
        for e in entries {
            let slot = totals.entry(e.day).or_insert(0);
            *slot = slot.saturating_add(e.count);
        }
    }
    totals
        .into_iter()
        .map(|(day, count)| DayCount { day, count })
        .collect()
}

/// What today still needs to hit `goal`, and whether it is already there.
///
/// Weekly and yearly goals are spread evenly across their period, so this is
/// the daily share rounded up: finishing the year needs every day to clear the
/// average, and rounding down would quietly let a goal slip.
pub fn remaining_today(goal: Goal, today_count: u32, days_in_year: u32) -> u32 {
    goal.daily_target(days_in_year).saturating_sub(today_count)
}

/// How much of `delta` a day holding `today_count` can actually take.
///
/// A day never goes below zero, so subtracting more than is there only takes
/// what is there, and subtracting from an empty day changes nothing. Callers
/// use this for both the store write and the message they show, so the two
/// always agree.
pub fn applied_delta(today_count: u32, delta: i64) -> i64 {
    if delta >= 0 {
        return delta;
    }
    // Negate first, then clamp, then negate back. Doing it in one step reads
    // as `-(delta.min(n))`, which turns a subtraction into an addition.
    let wanted = delta.saturating_neg();
    -wanted.min(i64::from(today_count))
}

/// One counter's inputs for the figures that span all of them.
#[derive(Debug, Clone, Copy)]
pub struct CounterDays<'a> {
    /// That counter's goal, if it has one.
    pub goal: Option<Goal>,
    /// Its whole history, sorted by day.
    pub entries: &'a [DayCount],
}

/// Today across every counter at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Across {
    /// Everything logged today, all counters summed.
    pub logged_today: u32,
    /// Every entry ever, all counters summed.
    pub lifetime: u32,
    /// Counters whose daily share is already met.
    pub goals_met: usize,
    /// Counters that have a goal at all, the denominator for `goals_met`.
    pub with_goals: usize,
    /// Consecutive days with something logged on any counter.
    pub streak: u32,
}

/// The home screen's headline figures.
///
/// The streak here is deliberately not the best of the per-counter streaks.
/// It merges every counter's history by day first, so it counts days you
/// logged anything at all, which is the run that is actually hard to break
/// and the one worth showing above a list.
///
/// Reading the entries is the caller's job; this only does the arithmetic,
/// which is what makes it testable.
pub fn across_counters(
    counters: &[CounterDays<'_>],
    today: NaiveDate,
    prefs: &Preferences,
) -> Across {
    let days = days_in_year(today.year());
    let mut out = Across::default();

    for c in counters {
        let s = summarize_with(c.entries, c.goal, today, prefs);
        out.logged_today = out.logged_today.saturating_add(s.today);
        out.lifetime = out.lifetime.saturating_add(s.lifetime);
        if let Some(g) = c.goal {
            out.with_goals += 1;
            if remaining_today(g, s.today, days) == 0 {
                out.goals_met += 1;
            }
        }
    }

    let merged = merge_days(counters.iter().map(|c| c.entries));
    out.streak = summarize_with(&merged, None, today, prefs).streak;
    out
}

/// Whether this tap is the one that finished the day.
///
/// True only when the day was short before and is not after. Firing from the
/// tap rather than from render state is what keeps this free: there is no
/// celebrated-today flag to store or clear, and reopening the app on a day
/// already finished celebrates nothing.
pub fn crosses_goal(goal: Goal, before: u32, applied: i64, days_in_year: u32) -> bool {
    if applied <= 0 {
        return false;
    }
    let after = u64::from(before).saturating_add(applied.unsigned_abs());
    let after = u32::try_from(after).unwrap_or(u32::MAX);
    remaining_today(goal, before, days_in_year) > 0
        && remaining_today(goal, after, days_in_year) == 0
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
        assert!((p.needed_per_day - 2650.0 / 266.0).abs() < 1e-9);

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
        assert!(p.needed_per_day > 10.0);
    }

    #[test]
    fn pace_past_the_goal_goes_negative() {
        let goal = Goal::per_year(10).unwrap();
        let p = pace(goal, 12, d(2026, 6, 1));
        assert_eq!(p.remaining, -2);
        assert!(p.needed_per_day < 0.0);
    }

    #[test]
    fn leap_year_uses_366() {
        let goal = Goal::per_year(366).unwrap();
        let p = pace(goal, 0, d(2028, 12, 31));
        assert_eq!(p.target_today, 366);
        assert!((p.needed_per_day - 366.0).abs() < f64::EPSILON);
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
        let p = Preferences::default();
        assert_eq!(streak(&entries, d(2026, 9, 19), &p), 3);
        // Today empty, yesterday counted: streak still alive.
        assert_eq!(streak(&entries, d(2026, 9, 20), &p), 3);
        // Two days gap: broken.
        assert_eq!(streak(&entries, d(2026, 9, 21), &p), 0);
        // Zero rows don't count.
        let with_zero = [e(2026, 9, 18, 0), e(2026, 9, 19, 5)];
        assert_eq!(streak(&with_zero, d(2026, 9, 19), &p), 1);
    }

    #[test]
    fn rest_days_neither_break_nor_extend_streaks() {
        use chrono::Weekday;
        // Fri 18, Sat 19, skip Sun 20 (rest), Mon 21.
        let entries = [e(2026, 9, 18, 1), e(2026, 9, 19, 1), e(2026, 9, 21, 1)];
        let p = Preferences::default().toggle_rest(Weekday::Sun);
        assert_eq!(streak(&entries, d(2026, 9, 21), &p), 3);
        assert_eq!(longest_streak(&entries, &p), 3);
        assert_eq!(longest_streak(&entries, &Preferences::default()), 2);
        // Consistency ignores rest days in the denominator: 3 of 3 counting days.
        let s = summarize_with(&entries, None, d(2026, 9, 21), &p);
        let counting = counting_days(2026, d(2026, 9, 21), &p);
        assert!(
            (s.consistency - 3.0 / f64::from(counting.min(s.this_year.days_elapsed))).abs() < 1e-9
        );
    }

    #[test]
    fn longest_streak_and_days_since_last() {
        let entries = [
            e(2026, 9, 1, 1),
            e(2026, 9, 2, 1),
            e(2026, 9, 3, 1),
            e(2026, 9, 10, 1),
        ];
        let s = summarize(&entries, None, d(2026, 9, 13));
        assert_eq!(s.longest_streak, 3);
        assert_eq!(s.streak, 0);
        assert_eq!(s.days_since_last, Some(3));
        assert!(
            summarize(&[], None, d(2026, 9, 13))
                .days_since_last
                .is_none()
        );
    }

    #[test]
    fn consistency_and_projection() {
        // 10 days in, logged 5 of them, 100 total: 10/day -> 3650 by year end.
        let entries: Vec<DayCount> = (1..=5).map(|day| e(2026, 1, day, 20)).collect();
        let s = summarize(&entries, None, d(2026, 1, 10));
        assert!((s.consistency - 0.5).abs() < f64::EPSILON);
        assert_eq!(s.projected_year_end, 3650);
    }

    #[test]
    fn best_day_prefers_earliest_on_tie() {
        let entries = [e(2026, 1, 1, 50), e(2026, 1, 2, 50), e(2026, 1, 3, 10)];
        let s = summarize(&entries, None, d(2026, 1, 3));
        assert_eq!(s.best_day, Some(e(2026, 1, 1, 50)));
    }

    #[test]
    fn crossing_the_goal_fires_once_on_the_tap_that_does_it() {
        let goal = Goal::per_day(100).unwrap();
        // The tap that lands on it, and the one that sails past it.
        assert!(crosses_goal(goal, 90, 10, 365));
        assert!(crosses_goal(goal, 90, 50, 365));
        // Short before and short after.
        assert!(!crosses_goal(goal, 10, 10, 365));
        // Already finished: every later tap is silent.
        assert!(!crosses_goal(goal, 100, 10, 365));
        assert!(!crosses_goal(goal, 250, 10, 365));
    }

    #[test]
    fn subtracting_never_celebrates() {
        let goal = Goal::per_day(100).unwrap();
        assert!(!crosses_goal(goal, 150, -10, 365));
        assert!(!crosses_goal(goal, 100, -1, 365));
        // A tap that applied nothing is not a crossing either.
        assert!(!crosses_goal(goal, 90, 0, 365));
    }

    #[test]
    fn a_weekly_goal_crosses_at_its_daily_share() {
        // 20,000 a week is 2,858 a day once rounded up.
        let goal = Goal::per_week(20_000).unwrap();
        assert!(!crosses_goal(goal, 2_800, 50, 365));
        assert!(crosses_goal(goal, 2_800, 58, 365));
    }

    #[test]
    fn subtracting_never_takes_more_than_the_day_holds() {
        assert_eq!(applied_delta(25, -10), -10);
        assert_eq!(applied_delta(4, -10), -4);
        assert_eq!(applied_delta(0, -10), 0);
    }

    #[test]
    fn adding_is_never_clamped() {
        assert_eq!(applied_delta(0, 10), 10);
        assert_eq!(applied_delta(999, 1), 1);
    }

    #[test]
    fn subtracting_stays_negative() {
        // The bug this guards: `-delta.min(n)` negates the whole min, so a
        // subtraction came back positive and the minus button added.
        for today in [0u32, 1, 5, 100] {
            for step in [1i64, 10, 1000] {
                assert!(applied_delta(today, -step) <= 0, "{today} {step}");
            }
        }
    }

    fn cd(goal: Option<Goal>, entries: &[DayCount]) -> CounterDays<'_> {
        CounterDays { goal, entries }
    }

    #[test]
    fn across_counters_sums_today_and_lifetime() {
        let a = [e(2026, 1, 1, 10), e(2026, 1, 2, 5)];
        let b = [e(2026, 1, 2, 7)];
        let got = across_counters(
            &[cd(None, &a), cd(None, &b)],
            d(2026, 1, 2),
            &Preferences::default(),
        );
        assert_eq!(got.logged_today, 12);
        assert_eq!(got.lifetime, 22);
    }

    #[test]
    fn only_counters_with_a_goal_count_toward_goals_met() {
        let met = [e(2026, 1, 2, 100)];
        let short = [e(2026, 1, 2, 3)];
        let none = [e(2026, 1, 2, 999)];
        let goal = Goal::per_day(100).unwrap();
        let got = across_counters(
            &[
                cd(Some(goal), &met),
                cd(Some(goal), &short),
                cd(None, &none),
            ],
            d(2026, 1, 2),
            &Preferences::default(),
        );
        assert_eq!(
            got.with_goals, 2,
            "the counter without a goal is not a denominator"
        );
        assert_eq!(got.goals_met, 1);
    }

    #[test]
    fn the_streak_spans_counters_rather_than_taking_the_best_one() {
        // Neither counter ran three days alone. Together they did.
        let a = [e(2026, 1, 1, 1), e(2026, 1, 2, 1)];
        let b = [e(2026, 1, 3, 1)];
        let today = d(2026, 1, 3);
        let got = across_counters(
            &[cd(None, &a), cd(None, &b)],
            today,
            &Preferences::default(),
        );
        assert_eq!(got.streak, 3);
        // The best single counter only reaches 2: a's run ends yesterday,
        // which still counts because today is allowed to be empty, and b has
        // just the one day.
        assert_eq!(summarize(&a, None, today).streak, 2);
        assert_eq!(summarize(&b, None, today).streak, 1);
    }

    #[test]
    fn across_nothing_is_all_zero() {
        let got = across_counters(&[], d(2026, 1, 2), &Preferences::default());
        assert_eq!(got, Across::default());
    }

    #[test]
    fn merging_sums_shared_days_and_keeps_lone_ones() {
        let a = [e(2026, 1, 1, 10), e(2026, 1, 3, 5)];
        let b = [e(2026, 1, 1, 7), e(2026, 1, 2, 1)];
        let merged = merge_days([a.as_slice(), b.as_slice()]);
        assert_eq!(
            merged,
            vec![e(2026, 1, 1, 17), e(2026, 1, 2, 1), e(2026, 1, 3, 5)]
        );
    }

    #[test]
    fn merging_nothing_is_empty() {
        assert!(merge_days(std::iter::empty()).is_empty());
    }

    #[test]
    fn a_merged_streak_counts_any_counter_logging() {
        // Neither counter alone ran three days; together they did.
        let a = [e(2026, 1, 1, 1), e(2026, 1, 3, 1)];
        let b = [e(2026, 1, 2, 1)];
        let merged = merge_days([a.as_slice(), b.as_slice()]);
        assert_eq!(summarize(&merged, None, d(2026, 1, 3)).streak, 3);
    }

    #[test]
    fn remaining_counts_down_to_met() {
        let goal = Goal::per_day(100).unwrap();
        assert_eq!(remaining_today(goal, 0, 365), 100);
        assert_eq!(remaining_today(goal, 40, 365), 60);
        assert_eq!(remaining_today(goal, 100, 365), 0);
        assert_eq!(remaining_today(goal, 250, 365), 0);
    }

    #[test]
    fn a_weekly_goal_spreads_over_seven_days() {
        // 20,000 a week is 2,857.14 a day, so the day is not done until 2,858.
        let goal = Goal::per_week(20_000).unwrap();
        assert_eq!(goal.daily_target(365), 2_858);
        assert_eq!(remaining_today(goal, 0, 365), 2_858);
        assert_eq!(remaining_today(goal, 2_857, 365), 1);
        assert_eq!(remaining_today(goal, 2_858, 365), 0);
        // An exact multiple of seven must not round up a spurious extra rep.
        assert_eq!(Goal::per_week(70).unwrap().daily_target(365), 10);
    }

    #[test]
    fn a_yearly_goal_rounds_its_daily_share_up() {
        // 1000/365 is 2.74 a day. Logging two would leave the year short, so
        // the day is not done until three.
        let goal = Goal::per_year(1000).unwrap();
        assert_eq!(remaining_today(goal, 0, 365), 3);
        assert_eq!(remaining_today(goal, 2, 365), 1);
        assert_eq!(remaining_today(goal, 3, 365), 0);
    }
}
