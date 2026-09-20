//! A label over a number, for the stat grids.

use dioxus::prelude::*;

/// One statistic. `value` is preformatted by the caller so the tile stays
/// dumb about units and precision. `hint` is an optional muted line under it.
#[component]
pub fn StatTile(label: String, value: String, #[props(default)] hint: Option<String>) -> Element {
    rsx! {
        div { class: "stat-tile",
            span { class: "stat-label", "{label}" }
            span { class: "stat-value", "{value}" }
            if let Some(h) = hint {
                span { class: "stat-hint", "{h}" }
            }
        }
    }
}

/// Re-exported so screens keep one import for the tile and its number style.
pub use crate::domain::counter::format::rate;
