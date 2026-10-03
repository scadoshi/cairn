//! The last year of one counter as a grid of days, a column per week, each
//! cell shaded by how the day ranks against the counter's other logged days.
//! Tapping a cell names the day and its count.

use crate::{
    domain::counter::{DayCount, format::thousands, heat},
    inbound::ui::{components::reveal::use_seen, today, use_date_format, use_prefs},
};
use dioxus::prelude::*;
use std::time::Duration;
use zwipe_components::Figure;

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

/// The grid, its caption, and the tap chip.
#[component]
pub fn Heatmap(entries: Vec<DayCount>) -> Element {
    let prefs = use_prefs()();
    let df = use_date_format()();
    let seen = use_seen();
    let mut tip: Signal<Option<Tip>> = use_signal(|| None);
    let grid = heat::heat_grid(&entries, today(), prefs.week_start, WEEKS);
    let labels = prefs.weekday_labels();
    let width = LEFT + px(grid.columns) * STEP;
    let height = TOP + 7.0 * STEP;
    let days = grid.cells.len();

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
                    for cell in grid.cells.iter() {
                        {
                            let x = LEFT + px(cell.column) * STEP;
                            let y = TOP + px(cell.row) * STEP;
                            let text = format!("{} on {}", thousands(cell.count), df.date(cell.day));
                            let (cx, cy) = ((x + CELL / 2.0) / width * 100.0, y / height * 100.0);
                            rsx! {
                                rect {
                                    key: "{cell.day}",
                                    class: "heat-cell heat-{cell.level}",
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
                if let Some(t) = tip() {
                    span {
                        class: "stat-chip heat-tip",
                        class: if t.left < 15.0 { "tip-start" } else if t.left > 85.0 { "tip-end" } else { "" },
                        style: "left: {t.left}%; top: {t.top}%;",
                        "{t.text}"
                    }
                }
            }
        }
        p { class: "chart-note heat-note",
            Figure { text: thousands(grid.total), start: seen }
            " across "
            Figure { text: grid.active_days.to_string(), start: seen }
            " of {days} days"
        }
    }
}
