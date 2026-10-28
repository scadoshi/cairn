//! The last year of one counter as a grid of days, a column per week, each
//! cell shaded by how the day ranks against the counter's other logged days,
//! on a field of the backdrop's tiles. Tapping a cell names the day and its
//! count: in a floating chip where the screen has room and a pointer, in the
//! readout under the grid otherwise.

use std::rc::Rc;

use dioxus::prelude::*;
use orangui::{
    ChartTip, HEAT_CELL, HEAT_ROWS, HEAT_STEP, HeatCell, HeatGrid, HeatHit, HeatReadout,
    use_heat_pick, use_scroll_to_end,
};

use crate::domain::counter::{DayCount, format, heat};
use crate::inbound::ui::{self, use_date_format, use_prefs};

/// Columns of tiles left of the grid, room for the weekday labels.
const FIELD_LEFT_COLUMNS: usize = 2;
/// Rows of tiles in the field: one over the grid for the month labels, the
/// seven weekdays, and one under it.
const FIELD_ROWS: usize = HEAT_ROWS + 2;
/// Room on the left for the weekday labels and on top for the months, in
/// whole tiles so the days land on the field's tiles.
const LEFT: f64 = 2.0 * HEAT_STEP;
const TOP: f64 = HEAT_STEP;
/// The backdrop's tile shades, as in orangui's `grid-cells.js`. The
/// stylesheet holds the alphas; this is how many there are.
const TILE_SHADES: usize = 5;
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
    for i in orangui::peak_indices(&counts, PEAK_RATIO, MAX_PEAKS) {
        if let Some(slot) = peak.get_mut(i) {
            *slot = true;
        }
    }
    peak
}

/// The field's width and height in SVG units for a year `columns` weeks
/// wide: the label room, the grid, and a column of tiles past its right edge
/// and a row under it.
fn field_size(columns: usize) -> (f64, f64) {
    (
        LEFT + orangui::heat_span(columns.saturating_add(1)),
        TOP + orangui::heat_span(HEAT_ROWS.saturating_add(1)),
    )
}

/// How many tile columns a year `columns` weeks wide needs.
fn tiles_across(columns: usize) -> usize {
    FIELD_LEFT_COLUMNS.saturating_add(columns).saturating_add(1)
}

/// A tile's shade, 0 to 4, picked by a hash of its place, so the field is
/// the same on every render.
fn tile_shade(column: usize, row: usize) -> usize {
    let mut h = column.wrapping_mul(2_654_435_761) ^ row.wrapping_mul(2_246_822_519);
    h ^= h >> 13;
    h = h.wrapping_mul(1_540_483_477);
    h ^= h >> 15;
    h % TILE_SHADES
}

/// The field's tiles, `across` columns of [`FIELD_ROWS`], each in one of
/// the backdrop's shades. They hold still; the days sweep in over them.
#[component]
fn HeatTiles(across: usize) -> Element {
    rsx! {
        for column in 0..across {
            for row in 0..FIELD_ROWS {
                rect {
                    key: "t{column}-{row}",
                    class: "heat-tile heat-tile-{tile_shade(column, row)}",
                    x: "{orangui::heat_span(column)}",
                    y: "{orangui::heat_span(row)}",
                    width: "{HEAT_CELL}",
                    height: "{HEAT_CELL}",
                    rx: "2",
                }
            }
        }
    }
}

/// The grid on its field of tiles, edge to edge in its card, then a rule
/// and the readout under it like the card's action bar, and the tap chip.
#[component]
pub fn Heatmap(entries: Vec<DayCount>) -> Element {
    let prefs = use_prefs()();
    let df = use_date_format()();
    let mut tip: Signal<Option<Tip>> = use_signal(|| None);
    let pick = use_heat_pick();
    let grid = heat::heat_grid(&entries, ui::today(), prefs.week_start, WEEKS);
    let labels = prefs.weekday_labels();
    let (width, height) = field_size(grid.columns);
    let days = grid.cells.len();
    // What each cell's chip and readout say, by the cell's index.
    let texts: Rc<[String]> = grid
        .cells
        .iter()
        .map(|cell| format!("{} on {}", format::thousands(cell.count), df.date(cell.day)))
        .collect();
    let tags: Rc<[Vec<String>]> = grid
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
        div { class: "heat-field",
            div { class: "chart-scroll scroll-end heat-scroll",
                div { class: "heat-plot",
                    svg {
                        class: "heat-grid",
                        view_box: "0 0 {width} {height}",
                        role: "img",
                        HeatTiles { across: tiles_across(grid.columns) }
                        // Clears the chip when the tap lands on nothing.
                        rect {
                            class: "heat-backdrop",
                            x: "0",
                            y: "0",
                            width: "{width}",
                            height: "{height}",
                            onclick: move |_| {
                                tip.set(None);
                                pick.clear();
                            },
                        }
                        for m in grid.months.iter() {
                            text { class: "heat-label", x: "{LEFT + orangui::heat_span(m.column)}", y: "{TOP - 4.0}", "{m.name}" }
                        }
                        for (row, label) in labels.iter().enumerate() {
                            if row % 2 == 1 {
                                text { class: "heat-label", x: "{LEFT - 4.0}", y: "{TOP + orangui::heat_span(row) + HEAT_CELL - 2.0}", text_anchor: "end", "{label}" }
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
                                if let Some(tags) = tags.get(hit.index) {
                                    pick.show(tags.clone());
                                }
                            },
                        }
                    }
                    if let Some(t) = tip() {
                        ChartTip {
                            text: t.text,
                            left: t.left,
                            top: t.top,
                            class: "stat-chip heat-float",
                        }
                    }
                }
            }
        }
        HeatReadout { rest, pick }
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

    #[test]
    fn days_land_on_whole_tiles_in_every_shade() {
        assert!((LEFT / HEAT_STEP - 2.0).abs() < f64::EPSILON);
        assert!((TOP / HEAT_STEP - 1.0).abs() < f64::EPSILON);
        let grid = heat::heat_grid(&[], d("2026-10-28"), Weekday::Mon, WEEKS);
        let (width, height) = field_size(grid.columns);
        let across = tiles_across(grid.columns);
        assert!((orangui::heat_span(across) - width).abs() < f64::EPSILON);
        assert!((orangui::heat_span(FIELD_ROWS) - height).abs() < f64::EPSILON);
        let mut seen = [false; TILE_SHADES];
        for column in 0..across {
            for row in 0..FIELD_ROWS {
                let shade = tile_shade(column, row);
                assert_eq!(shade, tile_shade(column, row));
                seen[shade] = true;
            }
        }
        assert!(seen.iter().all(|&s| s), "{seen:?}");
    }
}
