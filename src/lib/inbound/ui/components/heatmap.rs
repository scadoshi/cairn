//! The last year of one counter as a grid of days, a column per week, each
//! cell shaded by how the day ranks against the counter's other logged days.
//! Hovering or tapping a cell names the day and its count: in a floating chip
//! where the screen has room and a pointer, in the readout under the grid
//! otherwise. The grid itself is orangui's.

use dioxus::prelude::*;
use orangui::{HeatCell, Heatmap as YearHeat};

use crate::domain::counter::{DayCount, format, heat};
use crate::inbound::ui::{self, use_date_format, use_prefs};

/// How many weeks the grid shows.
const WEEKS: usize = 52;
/// A day at or past this many times the median logged day is a peak, and at
/// most this many of them, the biggest.
const PEAK_RATIO: u32 = 4;
const MAX_PEAKS: usize = 12;

/// Which cells are the year's outlier days, in cell order: at or past
/// `PEAK_RATIO` times the median logged day, among the `MAX_PEAKS` biggest.
fn peaks(cells: &[heat::HeatCell]) -> Vec<bool> {
    let counts: Vec<u32> = cells.iter().map(|c| c.count).collect();
    let mut peak = vec![false; cells.len()];
    for i in orangui::peak_indices(&counts, PEAK_RATIO, MAX_PEAKS) {
        if let Some(slot) = peak.get_mut(i) {
            *slot = true;
        }
    }
    peak
}

/// The odd rows' weekday names, every other weekday from the second, for a
/// week that starts on `labels`' first day.
fn odd_weekdays(labels: &[&str]) -> Vec<(usize, String)> {
    labels
        .iter()
        .enumerate()
        .filter(|(row, _)| row % 2 == 1)
        .map(|(row, label)| (row, (*label).to_owned()))
        .collect()
}

/// The counter's last year in orangui's heatmap, in Cairn's words: a day's
/// chip says its count and date, its readout tags the same, and the resting
/// tags the year's total and how many days were logged.
#[component]
pub fn Heatmap(entries: Vec<DayCount>) -> Element {
    let prefs = use_prefs()();
    let df = use_date_format()();
    let grid = heat::heat_grid(&entries, ui::shown_day(), prefs.week_start, WEEKS);
    let days = grid.cells.len();
    let chips: Vec<String> = grid
        .cells
        .iter()
        .map(|cell| format!("{} on {}", format::thousands(cell.count), df.date(cell.day)))
        .collect();
    let tags: Vec<Vec<String>> = grid
        .cells
        .iter()
        .map(|cell| {
            vec![
                format!("{} logged", format::thousands(cell.count)),
                df.date(cell.day),
            ]
        })
        .collect();
    let rest = vec![
        format!("{} logged", format::thousands(grid.total)),
        format!("{} of {days} days", grid.active_days),
    ];
    let label = format!("The last year by day: {}", rest.join(", "));
    let cells: Vec<HeatCell> = grid
        .cells
        .iter()
        .zip(peaks(&grid.cells))
        .map(|(cell, peak)| HeatCell {
            column: cell.column,
            row: cell.row,
            level: cell.level,
            peak,
            key: cell.day.to_string(),
        })
        .collect();
    let months: Vec<(usize, String)> = grid
        .months
        .iter()
        .map(|m| (m.column, m.name.to_owned()))
        .collect();
    rsx! {
        YearHeat {
            cells,
            columns: grid.columns,
            months,
            weekdays: odd_weekdays(&prefs.weekday_labels()),
            chips,
            tags,
            rest,
            label,
            chip_class: "stat-chip",
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration as Days, NaiveDate, Weekday};

    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn the_odd_rows_are_named() {
        let names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        assert_eq!(
            odd_weekdays(&names),
            [
                (1, "Tue".to_owned()),
                (3, "Thu".to_owned()),
                (5, "Sat".to_owned())
            ]
        );
    }

    #[test]
    fn a_spike_is_a_peak_and_a_steady_stretch_is_not() {
        let mut entries: Vec<DayCount> = (1..=20u32)
            .map(|i| DayCount {
                day: d("2026-09-01") + Days::days(i64::from(i)),
                count: 10,
            })
            .collect();
        entries.push(DayCount {
            day: d("2026-09-25"),
            count: 45,
        });
        let grid = heat::heat_grid(&entries, d("2026-10-03"), Weekday::Mon, WEEKS);
        let peak_days: Vec<NaiveDate> = grid
            .cells
            .iter()
            .zip(peaks(&grid.cells))
            .filter(|(_, peak)| *peak)
            .map(|(c, _)| c.day)
            .collect();
        assert_eq!(peak_days, [d("2026-09-25")]);
    }
}
