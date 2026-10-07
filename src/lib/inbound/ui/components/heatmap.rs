//! The last year of one counter as a grid of days, a column per week, each
//! cell shaded by how the day ranks against the counter's other logged days.
//! Tapping a cell names the day and its count.

use crate::{
    domain::counter::{
        DayCount,
        format::thousands,
        heat::{self, HeatCell},
    },
    inbound::ui::{components::tile::Num, today, use_date_format, use_prefs},
};
use dioxus::prelude::*;
use std::time::Duration;
use zwipe_components::{peak_indices, tip_anchor};

/// Cell size and the gap between cells, in SVG units.
const CELL: f64 = 11.0;
const GAP: f64 = 2.0;
const STEP: f64 = CELL + GAP;
/// Room on the left for the weekday labels and on top for the months.
const LEFT: f64 = 26.0;
const TOP: f64 = 14.0;
/// How far apart the columns arrive in the sweep.
const SWEEP_STEP_MS: usize = 12;
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

/// Cell counts are tiny, so the conversion is exact.
#[allow(clippy::cast_precision_loss)]
fn px(n: usize) -> f64 {
    n as f64
}

/// Which cells are the year's outlier days, in cell order: at or past
/// `PEAK_RATIO` times the median logged day, among the `MAX_PEAKS` biggest.
fn peaks(cells: &[HeatCell]) -> Vec<bool> {
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
    let width = LEFT + px(grid.columns) * STEP;
    let height = TOP + 7.0 * STEP;
    let days = grid.cells.len();
    let peak = peaks(&grid.cells);

    // Opens on the newest weeks: the grid is wider than a phone, and the
    // interesting end is the right one.
    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(60)).await;
            let _ = document::eval(
                "for (const el of document.querySelectorAll('.heat-scroll')) el.scrollLeft = el.scrollWidth;",
            )
            .await;
        });
    });

    rsx! {
        div { class: "heat-scroll",
            div { class: "heat-plot",
                svg {
                    class: "heat-grid",
                    view_box: "0 0 {width} {height}",
                    role: "img",
                    // Clears the chip when the tap lands on nothing.
                    rect { class: "heat-backdrop", x: "0", y: "0", width: "{width}", height: "{height}", onclick: move |_| tip.set(None) }
                    for m in grid.months.iter() {
                        text { class: "heat-label", x: "{LEFT + px(m.column) * STEP}", y: "{TOP - 4.0}", "{m.name}" }
                    }
                    for (row, label) in labels.iter().enumerate() {
                        if row % 2 == 1 {
                            text { class: "heat-label", x: "{LEFT - 4.0}", y: "{TOP + px(row) * STEP + CELL - 2.0}", text_anchor: "end", "{label}" }
                        }
                    }
                    for (cell, peak) in grid.cells.iter().zip(peak) {
                        {
                            let x = LEFT + px(cell.column) * STEP;
                            let y = TOP + px(cell.row) * STEP;
                            let text = format!("{} on {}", thousands(cell.count), df.date(cell.day));
                            let (cx, cy) = ((x + CELL / 2.0) / width * 100.0, y / height * 100.0);
                            rsx! {
                                g { key: "{cell.day}",
                                // The glow behind a peak day is a shape, since iOS
                                // Safari applies no CSS filter to an SVG child.
                                if peak {
                                    rect {
                                        class: "heat-halo",
                                        x: "{x - 2.5}",
                                        y: "{y - 2.5}",
                                        width: "{CELL + 5.0}",
                                        height: "{CELL + 5.0}",
                                        rx: "4",
                                        style: "animation-delay: {cell.column * SWEEP_STEP_MS}ms",
                                    }
                                }
                                rect {
                                    class: if peak { "heat-cell heat-{cell.level} heat-peak" } else { "heat-cell heat-{cell.level}" },
                                    x: "{x}",
                                    y: "{y}",
                                    width: "{CELL}",
                                    height: "{CELL}",
                                    rx: "2",
                                    style: "animation-delay: {cell.column * SWEEP_STEP_MS}ms",
                                    onclick: move |_| tip.set(Some(Tip { text: text.clone(), left: cx, top: cy })),
                                }
                                }
                            }
                        }
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
