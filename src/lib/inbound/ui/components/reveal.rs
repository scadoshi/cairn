//! A card that arrives when it scrolls into view rather than when it mounts.
//!
//! The arriving is done by `assets/entrance.js`, inside the page: it watches
//! every `.reveal`, swaps `reveal-pending` for `reveal-in` as the viewport
//! reaches one, and rolls the `.figure` spans inside it. iOS holds the app's
//! messages to the page until a scroll settles, so this cannot be decided
//! over here without arriving late.

use dioxus::prelude::*;

/// A card, with `class` on it, that the page reveals on arrival. Without the
/// script it is simply visible.
#[component]
pub fn Reveal(#[props(default)] class: String, children: Element) -> Element {
    rsx! {
        div { class: "{class} reveal", {children} }
    }
}
