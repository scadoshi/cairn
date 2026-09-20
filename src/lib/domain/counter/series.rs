//! Series behind the trend charts. Pure functions over entries and events,
//! with `today` or a year passed in; the UI picks which to draw.

use super::{DayCount, Event};
use chrono::{Datelike, Duration, NaiveDate, Timelike, Weekday};

/// One count per day for every day from `from` to `to` inclusive, zero where
/// nothing was logged, so the line has a point for every day.
pub fn daily(entries: &[DayCount], from: NaiveDate, to: NaiveDate) -> Vec<(NaiveDate, u32)> {
    let mut out = Vec::new();
    let mut day = from;
    while day <= to {
        let count = entries.iter().find(|e| e.day == day).map_or(0, |e| e.count);
        out.push((day, count));
        match day.succ_opt() {
            Some(next) => day = next,
            None => break,
        }
    }
    out
}

/// Trailing average over the last `window` points (fewer at the start), the
/// smoothed line that shows the trend under the daily noise.
pub fn rolling_average(series: &[(NaiveDate, u32)], window: usize) -> Vec<(NaiveDate, f64)> {
    let window = window.max(1);
    series
        .iter()
        .enumerate()
        .map(|(i, (day, _))| {
            let start = i.saturating_sub(window - 1);
            let slice = series.get(start..=i).unwrap_or(&[]);
            let sum: u32 = slice.iter().map(|(_, c)| c).sum();
            let n = slice.len().max(1);
            #[allow(clippy::cast_precision_loss)]
            let avg = f64::from(sum) / n as f64;
            (*day, avg)
        })
        .collect()
}

/// The Monday of the week containing `day`.
pub fn week_start(day: NaiveDate) -> NaiveDate {
    day - Duration::days(i64::from(day.weekday().num_days_from_monday()))
}

/// Counts for Monday through Sunday of the week containing `today`; days
/// after today are `None` so the line stops at the present.
pub fn this_week(entries: &[DayCount], today: NaiveDate) -> [Option<u32>; 7] {
    let monday = week_start(today);
    let mut out = [None; 7];
    for (i, slot) in (0i64..).zip(out.iter_mut()) {
        let day = monday + Duration::days(i);
        if day <= today {
            *slot = Some(entries.iter().find(|e| e.day == day).map_or(0, |e| e.count));
        }
    }
    out
}

/// Total for the seven days ending `end` inclusive.
pub fn window_total(entries: &[DayCount], end: NaiveDate, days: i64) -> u32 {
    let start = end - Duration::days(days - 1);
    entries
        .iter()
        .filter(|e| e.day >= start && e.day <= end)
        .fold(0u32, |acc, e| acc.saturating_add(e.count))
}

/// Average count per active day for each weekday, Monday first, over all
/// entries. `None` for a weekday never logged.
pub fn weekday_average(entries: &[DayCount]) -> [Option<f64>; 7] {
    let mut sums = [0u32; 7];
    let mut days = [0u32; 7];
    for e in entries.iter().filter(|e| e.count > 0) {
        let i = e.day.weekday().num_days_from_monday() as usize;
        if let (Some(s), Some(d)) = (sums.get_mut(i), days.get_mut(i)) {
            *s = s.saturating_add(e.count);
            *d += 1;
        }
    }
    let mut out = [None; 7];
    for ((o, sum), n) in out.iter_mut().zip(sums).zip(days) {
        if n > 0 {
            *o = Some(f64::from(sum) / f64::from(n));
        }
    }
    out
}

/// Short weekday names, Monday first, for chart labels.
pub const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// The weekday of `day`, as an index into [`WEEKDAYS`].
pub fn weekday_index(day: NaiveDate) -> usize {
    match day.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

/// Total per ISO week, keyed by the Monday, for the weeks touching `year`.
pub fn weekly_totals(entries: &[DayCount], year: i32) -> Vec<(NaiveDate, u32)> {
    let mut out: Vec<(NaiveDate, u32)> = Vec::new();
    for e in entries.iter().filter(|e| e.day.year() == year) {
        let monday = week_start(e.day);
        match out.iter_mut().find(|(m, _)| *m == monday) {
            Some((_, total)) => *total = total.saturating_add(e.count),
            None => out.push((monday, e.count)),
        }
    }
    out.sort_by_key(|(m, _)| *m);
    out
}

/// Total and active-day count for each calendar month of `year`. `None`
/// where nothing was logged that month.
pub fn monthly_totals(entries: &[DayCount], year: i32) -> [Option<(u32, u32)>; 12] {
    let mut sums = [0u32; 12];
    let mut days = [0u32; 12];
    for e in entries
        .iter()
        .filter(|e| e.day.year() == year && e.count > 0)
    {
        let m = (e.day.month0()) as usize;
        if let (Some(s), Some(d)) = (sums.get_mut(m), days.get_mut(m)) {
            *s = s.saturating_add(e.count);
            *d += 1;
        }
    }
    let mut out = [None; 12];
    for ((o, sum), n) in out.iter_mut().zip(sums).zip(days) {
        if n > 0 {
            *o = Some((sum, n));
        }
    }
    out
}

/// Average count per active day, for each calendar month of `year`. `None`
/// where nothing was logged that month, so the line can skip it.
pub fn monthly_average(entries: &[DayCount], year: i32) -> [Option<f64>; 12] {
    let mut sums = [0u32; 12];
    let mut days = [0u32; 12];
    for e in entries
        .iter()
        .filter(|e| e.day.year() == year && e.count > 0)
    {
        let m = (e.day.month0()) as usize;
        if let (Some(s), Some(d)) = (sums.get_mut(m), days.get_mut(m)) {
            *s = s.saturating_add(e.count);
            *d += 1;
        }
    }
    let mut out = [None; 12];
    for ((o, sum), n) in out.iter_mut().zip(sums).zip(days) {
        if n > 0 {
            *o = Some(f64::from(sum) / f64::from(n));
        }
    }
    out
}

/// What an hourly average is divided by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HourlyBasis {
    /// Days with at least one tap. A rest day doesn't pull the curve down,
    /// so it reads as "on a day I train, when do I train".
    ActiveDays,
    /// Every calendar day in the span, rest days included, so it reads as
    /// "how many reps does a typical day hold at this hour".
    AllDays {
        /// Length of the span in days, at least 1.
        days: u32,
    },
}

/// Average reps logged in each hour of the day: the sum of positive deltas
/// in that hour divided by the day count `basis` picks. Shows when in the
/// day the reps happen.
pub fn hourly_average(events: &[Event], basis: HourlyBasis) -> [f64; 24] {
    let mut sums = [0i64; 24];
    let mut active: Vec<NaiveDate> = Vec::new();
    for e in events {
        let h = e.at.hour() as usize;
        if let Some(s) = sums.get_mut(h) {
            *s += e.delta.max(0);
        }
        let d = e.at.date();
        if !active.contains(&d) {
            active.push(d);
        }
    }
    let n = match basis {
        HourlyBasis::ActiveDays => active.len().max(1),
        HourlyBasis::AllDays { days } => days.max(1) as usize,
    };
    let mut out = [0.0; 24];
    for (o, sum) in out.iter_mut().zip(sums) {
        #[allow(clippy::cast_precision_loss)]
        let avg = sum as f64 / n as f64;
        *o = avg;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }
    fn e(y: i32, m: u32, day: u32, count: u32) -> DayCount {
        DayCount {
            day: d(y, m, day),
            count,
        }
    }
    fn ev(y: i32, m: u32, day: u32, h: u32, delta: i64) -> Event {
        Event {
            at: d(y, m, day).and_time(NaiveTime::from_hms_opt(h, 0, 0).unwrap()),
            delta,
        }
    }

    #[test]
    fn this_week_stops_at_today() {
        // 2026-09-16 is a Wednesday.
        let w = this_week(&[e(2026, 9, 14, 5), e(2026, 9, 16, 7)], d(2026, 9, 16));
        assert_eq!(w, [Some(5), Some(0), Some(7), None, None, None, None]);
    }

    #[test]
    fn window_total_is_inclusive() {
        let t = window_total(
            &[e(2026, 9, 10, 1), e(2026, 9, 11, 2), e(2026, 9, 17, 4)],
            d(2026, 9, 17),
            7,
        );
        assert_eq!(t, 6);
    }

    #[test]
    fn weekday_average_groups_by_weekday() {
        // 14th and 21st are Mondays.
        let w = weekday_average(&[e(2026, 9, 14, 10), e(2026, 9, 21, 20), e(2026, 9, 15, 3)]);
        assert!((w[0].unwrap() - 15.0).abs() < f64::EPSILON);
        assert!((w[1].unwrap() - 3.0).abs() < f64::EPSILON);
        assert!(w[2].is_none());
        assert_eq!(weekday_index(d(2026, 9, 20)), 6);
    }

    #[test]
    fn daily_fills_gaps_with_zero() {
        let s = daily(
            &[e(2026, 1, 1, 5), e(2026, 1, 3, 7)],
            d(2026, 1, 1),
            d(2026, 1, 3),
        );
        assert_eq!(
            s,
            vec![(d(2026, 1, 1), 5), (d(2026, 1, 2), 0), (d(2026, 1, 3), 7)]
        );
    }

    #[test]
    fn rolling_average_trails() {
        let s = vec![(d(2026, 1, 1), 2), (d(2026, 1, 2), 4), (d(2026, 1, 3), 6)];
        let r = rolling_average(&s, 2);
        assert!((r[0].1 - 2.0).abs() < f64::EPSILON);
        assert!((r[1].1 - 3.0).abs() < f64::EPSILON);
        assert!((r[2].1 - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn weekly_groups_by_monday() {
        // 2026-01-05 is a Monday.
        let s = weekly_totals(
            &[e(2026, 1, 5, 1), e(2026, 1, 7, 2), e(2026, 1, 12, 3)],
            2026,
        );
        assert_eq!(s, vec![(d(2026, 1, 5), 3), (d(2026, 1, 12), 3)]);
    }

    #[test]
    fn monthly_totals_pair_total_with_active_days() {
        let m = monthly_totals(
            &[e(2026, 1, 1, 10), e(2026, 1, 2, 20), e(2026, 3, 1, 5)],
            2026,
        );
        assert_eq!(m[0], Some((30, 2)));
        assert_eq!(m[1], None);
        assert_eq!(m[2], Some((5, 1)));
    }

    #[test]
    fn monthly_average_skips_empty_months() {
        let m = monthly_average(
            &[e(2026, 1, 1, 10), e(2026, 1, 2, 20), e(2026, 3, 1, 5)],
            2026,
        );
        assert!((m[0].unwrap() - 15.0).abs() < f64::EPSILON);
        assert!(m[1].is_none());
        assert!((m[2].unwrap() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hourly_average_over_all_days_counts_rest_days() {
        let evs = [ev(2026, 1, 1, 7, 10), ev(2026, 1, 2, 7, 20)];
        let h = hourly_average(&evs, HourlyBasis::AllDays { days: 4 });
        assert!((h[7] - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn hourly_average_is_per_active_day_and_ignores_undo() {
        let evs = [
            ev(2026, 1, 1, 7, 10),
            ev(2026, 1, 1, 7, -1),
            ev(2026, 1, 2, 7, 20),
            ev(2026, 1, 2, 18, 4),
        ];
        let h = hourly_average(&evs, HourlyBasis::ActiveDays);
        assert!((h[7] - 15.0).abs() < f64::EPSILON);
        assert!((h[18] - 2.0).abs() < f64::EPSILON);
        assert!(h[0].abs() < f64::EPSILON);
    }
}
