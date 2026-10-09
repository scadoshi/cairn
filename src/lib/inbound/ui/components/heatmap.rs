//! The last year of one counter as a grid of days, a column per week, each
//! cell shaded by how the day ranks against the counter's other logged days.
//! Tapping a cell names the day and its count.

use crate::{
    domain::counter::{DayCount, format::thousands, heat},
    inbound::ui::{components::tile::Num, today, use_date_format, use_prefs},
};
use dioxus::prelude::*;
use std::rc::Rc;
use zwipe_components::{
    HEAT_CELL, HEAT_ROWS, HeatCell, HeatGrid, HeatHit, heat_span, peak_indices, tip_anchor,
    use_scroll_to_end,
};

/// Room on the left for the weekday labels and on top for the months.
const LEFT: f64 = 26.0;
const TOP: f64 = 14.0;
/// How many weeks the grid shows.
const WEEKS: usize = 52;
/// A day at or past this many times the median logged day is a peak, and at
/// most this many of them, the biggest.
const PEAK_RATIO: u32 = 4;
const MAX_PEAKS: usize = 12;

/// What the tapped cell says and where the chip sits, as percentages of the
/// grid so it follows the grid at any width.
#[derive(Clone, PartialEq)]
struct Tip {
    text: String,
    left: f64,
    top: f64,
}

/// Which cells are the year's outlier days, in cell order: at or past
/// `PEAK_RATIO` times the median logged day, among the `MAX_PEAKS` biggest.
fn peaks(cells: &[heat::HeatCell]) -> Vec<bool> {
    let counts: Vec<u32> = cells.iter().map(|c| c.count).collect();
    let mut peak = vec![false; cells.len()];
    for i in peak_indices(&counts, PEAK_RATIO, MAX_PEAKS) {
        if let Some(slot) = peak.get_mut(i) {
            *slot = true;
        }
    }
    peak
}

/// The grid, its caption, and the tap chip.
#[component]
pub fn Heatmap(entries: Vec<DayCount>) -> Element {
    let prefs = use_prefs()();
    let df = use_date_format()();
    let mut tip: Signal<Option<Tip>> = use_signal(|| None);
    let grid = heat::heat_grid(&entries, today(), prefs.week_start, WEEKS);
    let labels = prefs.weekday_labels();
    let width = LEFT + heat_span(grid.columns);
    let height = TOP + heat_span(HEAT_ROWS);
    let days = grid.cells.len();
    // What each cell's chip says, by the cell's index.
    let texts: Rc<[String]> = grid
        .cells
        .iter()
        .map(|cell| format!("{} on {}", thousands(cell.count), df.date(cell.day)))
        .collect();
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

    // Opens on the newest weeks: the grid is wider than a phone, and the
    // interesting end is the right one.
    use_scroll_to_end();

    rsx! {
        div { class: "chart-scroll scroll-end heat-scroll",
            div { class: "heat-plot",
                svg {
                    class: "heat-grid",
                    view_box: "0 0 {width} {height}",
                    role: "img",
                    // Clears the chip when the tap lands on nothing.
                    rect { class: "heat-backdrop", x: "0", y: "0", width: "{width}", height: "{height}", onclick: move |_| tip.set(None) }
                    for m in grid.months.iter() {
                        text { class: "heat-label", x: "{LEFT + heat_span(m.column)}", y: "{TOP - 4.0}", "{m.name}" }
                    }
                    for (row, label) in labels.iter().enumerate() {
                        if row % 2 == 1 {
                            text { class: "heat-label", x: "{LEFT - 4.0}", y: "{TOP + heat_span(row) + HEAT_CELL - 2.0}", text_anchor: "end", "{label}" }
                        }
                    }
                    HeatGrid {
                        cells,
                        left: LEFT,
                        top: TOP,
                        on_tap: move |hit: HeatHit| {
                            if let Some(text) = texts.get(hit.index) {
                                let left = (hit.x + HEAT_CELL / 2.0) / width * 100.0;
                                let top = hit.y / height * 100.0;
                                tip.set(Some(Tip { text: text.clone(), left, top }));
                            }
                        },
                    }
                }
                if let Some(t) = tip() {
                    span {
                        class: "stat-chip heat-tip {tip_anchor(t.left)}",
                        style: "left: {t.left}%; top: {t.top}%;",
                        "{t.text}"
                    }
                }
            }
        }
        p { class: "chart-note heat-note",
            Num { text: thousands(grid.total) }
            " across "
            Num { text: grid.active_days.to_string() }
            " of {days} days"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration as Days, NaiveDate, Weekday};

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
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
