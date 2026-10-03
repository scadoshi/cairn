//! A card that arrives when it scrolls into view rather than when it mounts,
//! and tells the figures inside it when that is, so a card below the fold
//! rolls its numbers as you reach it instead of unseen at load.

use dioxus::prelude::*;

/// Whether the nearest `Reveal` has come into view. The tiles read it to
/// hold their roll; a figure outside any `Reveal` rolls on mount.
#[derive(Clone, Copy)]
pub struct Seen(pub Signal<bool>);

/// The start signal for a figure: the enclosing card's, if there is one.
pub fn use_seen() -> Option<Signal<bool>> {
    try_consume_context::<Seen>().map(|seen| seen.0)
}

/// A card, with `class` on it, that holds its entrance (and every animation
/// inside it) until the viewport reaches it. Cards already on screen at
/// mount pass straight through.
#[component]
pub fn Reveal(#[props(default)] class: String, children: Element) -> Element {
    let mut seen = use_signal(|| false);
    use_context_provider(|| Seen(seen));
    rsx! {
        div {
            class: if seen() { "{class} reveal reveal-in" } else { "{class} reveal reveal-pending" },
            onvisible: move |evt| {
                if !seen() && evt.data().is_intersecting().unwrap_or(true) {
                    seen.set(true);
                }
            },
            {children}
        }
    }
}
