//! A readout: one number in a small box with its name under it, three to a
//! row. The counter's odometer in miniature, for the stats that are a single
//! figure. Anything that needs a sentence stays a ruled row.

use dioxus::prelude::*;

/// Re-exported so screens keep one import for the tile and its number style.
pub use crate::domain::counter::format::rate;

/// One readout. `value` is preformatted by the caller so the tile stays dumb
/// about units and precision. `hint` is an optional small line under the
/// label, for a qualifier like "606 ahead".
#[component]
pub fn Tile(label: String, value: String, #[props(default)] hint: Option<String>) -> Element {
    rsx! {
        div { class: "tile",
            span { class: "tile-value", "{value}" }
            span { class: "tile-label", "{label}" }
            if let Some(h) = hint {
                span { class: "tile-hint", "{h}" }
            }
        }
    }
}

/// Three tiles to a row, wrapping.
#[component]
pub fn TileGrid(children: Element) -> Element {
    rsx! {
        div { class: "tile-grid", {children} }
    }
}
