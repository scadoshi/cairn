//! The last year of days as a grid for the heatmap: one cell per day, a
//! column per week, rows in the week's own order, and a level per cell from
//! how the day ranks against the counter's other logged days.

use super::DayCount;
use chrono::{Datelike, Duration, NaiveDate, Weekday};

/// One day on the grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatCell {
    /// The calendar day.
    pub day: NaiveDate,
    /// What was logged that day.
    pub count: u32,
    /// Week, oldest first.
    pub column: usize,
    /// Weekday, counted from the week's first day.
    pub row: usize,
    /// 0 for nothing logged, then 1 to 4 by quartile of the logged days.
    pub level: u8,
}

/// A month label and the column it sits over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatMonth {
    /// The week the month starts in.
    pub column: usize,
    /// Three letters.
    pub name: &'static str,
}

/// The grid and what it adds up to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeatGrid {
    /// Every day in range, oldest first.
    pub cells: Vec<HeatCell>,
    /// The labels along the top.
    pub months: Vec<HeatMonth>,
    /// How many weeks wide.
    pub columns: usize,
    /// Everything logged in range.
    pub total: u32,
    /// Days in range with anything logged.
    pub active_days: usize,
}

/// Columns a month label needs before the next one, so two never touch.
const LABEL_SPAN: usize = 3;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// The `weeks` weeks ending on `today`, the first column starting on the
/// `week_start` at or before `today` minus that span, every day in between
/// present with its count or zero.
pub fn heat_grid(
    entries: &[DayCount],
    today: NaiveDate,
    week_start: Weekday,
    weeks: usize,
) -> HeatGrid {
    let span = i64::try_from(weeks.max(1) * 7 - 1).unwrap_or(363);
    let first = week_start_of(today - Duration::days(span), week_start);
    let thresholds = quartiles(entries, first, today);
    let mut cells = Vec::new();
    let mut months = Vec::new();
    let mut last_month = None;
    let mut total = 0u32;
    let mut active_days = 0usize;
    let mut day = first;
    while day <= today {
        let count = entries.iter().find(|e| e.day == day).map_or(0, |e| e.count);
        let since = (day - first).num_days();
        let column = usize::try_from(since / 7).unwrap_or(0);
        let row = usize::try_from(since % 7).unwrap_or(0);
        if row == 0 && Some(day.month()) != last_month {
            if months
                .last()
                .is_some_and(|m: &HeatMonth| column < m.column + LABEL_SPAN)
            {
                months.pop();
            }
            months.push(HeatMonth {
                column,
                name: MONTHS
                    .get(usize::try_from(day.month0()).unwrap_or(0))
                    .copied()
                    .unwrap_or(""),
            });
            last_month = Some(day.month());
        }
        total = total.saturating_add(count);
        if count > 0 {
            active_days += 1;
        }
        cells.push(HeatCell {
            day,
            count,
            column,
            row,
            level: level(count, thresholds),
        });
        match day.succ_opt() {
            Some(next) => day = next,
            None => break,
        }
    }
    let columns = cells.last().map_or(0, |c| c.column + 1);
    HeatGrid {
        cells,
        months,
        columns,
        total,
        active_days,
    }
}

/// The `week_start` on or before `day`.
fn week_start_of(day: NaiveDate, week_start: Weekday) -> NaiveDate {
    let offset = (7 + day.weekday().num_days_from_monday() - week_start.num_days_from_monday()) % 7;
    day - Duration::days(i64::from(offset))
}

/// The three cuts between the four levels: the counts of the logged days in
/// range at a quarter, a half and three quarters of the way up. All zero when
/// nothing is logged, so every day sits at level 0.
fn quartiles(entries: &[DayCount], from: NaiveDate, to: NaiveDate) -> [u32; 3] {
    let mut logged: Vec<u32> = entries
        .iter()
        .filter(|e| e.day >= from && e.day <= to && e.count > 0)
        .map(|e| e.count)
        .collect();
    if logged.is_empty() {
        return [0; 3];
    }
    logged.sort_unstable();
    let at = |q: usize| logged.get(logged.len() * q / 4).copied().unwrap_or(0);
    [at(1), at(2), at(3)]
}

/// Where `count` lands against the cuts.
fn level(count: u32, cuts: [u32; 3]) -> u8 {
    if count == 0 {
        0
    } else if count <= cuts[0] {
        1
    } else if count <= cuts[1] {
        2
    } else if count <= cuts[2] {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn the_grid_starts_on_a_week_start_and_ends_on_today() {
        // 2026-10-03 is a Saturday; a Monday week start a year back lands on
        // 2026-10-06 minus 364 days = 2025-10-04 (Saturday), whose Monday is
        // 2025-09-29.
        let grid = heat_grid(&[], d("2026-10-03"), Weekday::Mon, 52);
        assert_eq!(grid.cells.first().unwrap().day, d("2025-09-29"));
        assert_eq!(grid.cells.first().unwrap().row, 0);
        assert_eq!(grid.cells.last().unwrap().day, d("2026-10-03"));
        assert_eq!(
            grid.cells.last().unwrap().row,
            5,
            "Saturday is the sixth day of a Monday week"
        );
        assert_eq!(grid.columns, 53);
        assert!(grid.cells.iter().all(|c| c.level == 0));
        assert_eq!(grid.total, 0);
    }

    #[test]
    fn a_sunday_week_start_moves_the_rows() {
        let grid = heat_grid(&[], d("2026-10-03"), Weekday::Sun, 52);
        assert_eq!(grid.cells.first().unwrap().day.weekday(), Weekday::Sun);
        assert_eq!(grid.cells.last().unwrap().row, 6);
    }

    #[test]
    fn levels_follow_the_quartiles_of_the_logged_days() {
        let entries: Vec<DayCount> = (1..=8u32)
            .map(|i| DayCount {
                day: d("2026-09-01") + Duration::days(i64::from(i)),
                count: i * 10,
            })
            .collect();
        let grid = heat_grid(&entries, d("2026-10-03"), Weekday::Mon, 52);
        let level_of = |date: &str| grid.cells.iter().find(|c| c.day == d(date)).unwrap().level;
        assert_eq!(level_of("2026-09-02"), 1);
        assert_eq!(level_of("2026-09-04"), 1);
        assert_eq!(level_of("2026-09-05"), 2);
        assert_eq!(level_of("2026-09-07"), 3);
        assert_eq!(level_of("2026-09-09"), 4);
        assert_eq!(level_of("2026-10-01"), 0);
        assert_eq!(grid.total, 360);
        assert_eq!(grid.active_days, 8);
    }

    #[test]
    fn month_labels_do_not_touch() {
        let grid = heat_grid(&[], d("2026-10-03"), Weekday::Mon, 52);
        let columns: Vec<usize> = grid.months.iter().map(|m| m.column).collect();
        for pair in columns.windows(2) {
            assert!(pair[1] - pair[0] >= LABEL_SPAN, "{columns:?}");
        }
        assert_eq!(grid.months.last().unwrap().name, "Sep");
        assert!(grid.months.len() >= 11, "{:?}", grid.months);
    }
}
