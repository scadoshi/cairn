//! The flash when a counter's daily goal is met.
//!
//! Drawn at the app root beside the router, for the same reason dialogs are:
//! a `position: fixed` overlay inside a card is clipped by the card's rounded
//! corners, and one inside the scrolling column was trapped by its transform.
//! Both have already shipped as bugs.
//!
//! What fires it lives in `domain::counter::stats::crosses_goal`, called from
//! the tap handler rather than from render state, so it happens once on the
//! tap that crosses and never on a launch or a redraw.

use crate::domain::preferences::Celebration;
use dioxus::prelude::*;
use std::time::Duration;

/// How long the overlay stays mounted. Longer than either animation so the
/// last confetti piece lands before it goes.
const LINGER: Duration = Duration::from_millis(2200);

/// Confetti pieces: horizontal position, start delay in ms, fall duration in
/// ms, and which theme color. Precomputed so nothing needs a random number
/// generator, and uneven so it does not read as a machine pattern.
const PIECES: [(u32, u32, u32, u8); 18] = [
    (4, 0, 1500, 0),
    (11, 210, 1750, 1),
    (18, 90, 1350, 2),
    (25, 400, 1600, 3),
    (31, 60, 1900, 0),
    (38, 300, 1450, 1),
    (44, 150, 1700, 2),
    (51, 480, 1550, 3),
    (57, 30, 1800, 0),
    (63, 260, 1400, 1),
    (69, 120, 1650, 2),
    (75, 420, 1500, 3),
    (81, 190, 1850, 0),
    (86, 70, 1600, 1),
    (91, 350, 1450, 2),
    (95, 240, 1750, 3),
    (8, 520, 1550, 2),
    (98, 440, 1650, 0),
];

/// The slot. Provided by the app root.
#[derive(Clone, Copy)]
pub struct CelebrationHost(pub Signal<Option<Celebration>>);

/// Fires a celebration, if the setting wants one.
///
/// `Off` is checked here rather than at the call sites, so the tap handlers
/// stay about counting.
pub fn celebrate(host: CelebrationHost, how: Celebration) {
    if how == Celebration::Off {
        return;
    }
    let mut slot = host.0;
    slot.set(Some(how));
    spawn(async move {
        tokio::time::sleep(LINGER).await;
        // Only clear what we set: a second crossing during the linger
        // replaces this one rather than being cut short by it.
        if slot.peek().is_some() {
            slot.set(None);
        }
    });
}

/// Draws whatever is in the slot.
#[component]
pub fn CelebrationHostView() -> Element {
    let host = use_context::<CelebrationHost>();
    let Some(how) = host.0.read().to_owned() else {
        return rsx! {};
    };
    match how {
        Celebration::Off => rsx! {},
        Celebration::Sheen => rsx! {
            div { class: "celebrate-sheen", aria_hidden: "true" }
        },
        Celebration::Confetti => rsx! {
            div { class: "celebrate-confetti", aria_hidden: "true",
                for (i, (x, delay, dur, color)) in PIECES.iter().enumerate() {
                    div {
                        key: "{i}",
                        class: "confetti-piece confetti-color-{color}",
                        style: "left: {x}%; animation-delay: {delay}ms; animation-duration: {dur}ms;",
                    }
                }
            }
        },
    }
}
