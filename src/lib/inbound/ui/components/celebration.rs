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

use crate::domain::preferences::{Celebration, done_line};
use dioxus::prelude::*;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

/// How long the overlay stays mounted, per kind. The sheen is a single fast
/// pass; the confetti has to wait for the last piece to land.
const fn linger(how: Celebration) -> Duration {
    match how {
        Celebration::Sheen => Duration::from_millis(700),
        Celebration::Poppers => Duration::from_millis(2400),
        _ => Duration::from_millis(2200),
    }
}

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

/// How many celebrations have fired this run.
///
/// The line used to be picked from a figure read off the screen, which a
/// `use_callback` captured on first render and then never changed, so the
/// same line came up until the app was relaunched. A counter that the act of
/// celebrating bumps cannot go stale that way.
static FIRED: AtomicU64 = AtomicU64::new(0);

/// A success line, different from the one before it.
///
/// Rotation rather than randomness: random would sometimes hand you the same
/// line twice running, which is the one thing a rotating message must not do.
pub fn next_success_line() -> &'static str {
    done_line(FIRED.fetch_add(1, Ordering::Relaxed))
}

/// Popper pieces: how far across and up the piece travels as a percentage
/// of the screen, how much it spins, its delay, and which theme color.
/// Mirrored for the other corner, so one table covers both barrels.
const POPS: [(i32, i32, i32, u32, u8); 14] = [
    (18, 62, 520, 0, 0),
    (30, 74, 380, 10, 1),
    (44, 58, 640, 22, 2),
    (55, 40, 300, 8, 3),
    (24, 80, 700, 34, 1),
    (38, 68, 450, 18, 0),
    (62, 30, 560, 44, 2),
    (14, 46, 260, 26, 3),
    (48, 76, 620, 4, 0),
    (34, 34, 340, 52, 2),
    (58, 66, 480, 14, 1),
    (22, 28, 700, 40, 3),
    (52, 54, 400, 30, 1),
    (28, 50, 580, 48, 0),
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
        tokio::time::sleep(linger(how)).await;
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
        Celebration::Poppers => rsx! {
            div { class: "celebrate-poppers", aria_hidden: "true",
                // Two barrels from the bottom corners. The x sign is what
                // mirrors the table; everything else is shared.
                for (side, dir) in [("left", 1_i32), ("right", -1_i32)] {
                    div { class: "popper popper-{side}",
                        for (i, (dx, dy, spin, delay, color)) in POPS.iter().enumerate() {
                            div {
                                key: "{side}-{i}",
                                class: "popper-arc",
                                style: "--dx: {dx * dir}vw; animation-delay: {delay}ms;",
                                div {
                                    class: "popper-bit confetti-color-{color}",
                                    style: "--dy: -{dy}vh; --spin: {spin}deg; animation-delay: {delay}ms;",
                                }
                            }
                        }
                    }
                }
            }
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
