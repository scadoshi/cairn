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
        Celebration::Scanline => Duration::from_millis(850),
        Celebration::Pulse => Duration::from_millis(900),
        Celebration::Stamp => Duration::from_millis(1400),
        Celebration::Typewriter => Duration::from_millis(1900),
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

/// What is currently playing: the resolved animation and the line that goes
/// with it, since two of them draw the line themselves.
#[derive(Clone, PartialEq)]
pub struct Playing {
    /// Never `Random`; that is resolved before it gets here.
    pub how: Celebration,
    /// The success line for this crossing.
    pub line: &'static str,
    /// Which of the theme's three accents this one is drawn in, 0 to 2.
    /// The success green on everything made the animations look like
    /// variations of one effect rather than different effects.
    pub tint: u8,
    /// Which crossing this is. Used as the element key, so a re-render
    /// while an animation is running reuses the node instead of building a
    /// fresh one and starting the animation over.
    pub id: u64,
}

/// The slot. Provided by the app root.
#[derive(Clone, Copy)]
pub struct CelebrationHost(pub Signal<Option<Playing>>);

/// Fires a celebration, if the setting wants one.
///
/// Returns the line to raise in a toast, or `None` when the animation is
/// already showing those words, or when the setting is `Off`. Picking the
/// line here rather than at the call sites is what keeps the toast and the
/// animation saying the same thing.
///
/// `Off` is checked here too, so the tap handlers stay about counting.
pub fn celebrate(host: CelebrationHost, how: Celebration) -> Option<&'static str> {
    if how == Celebration::Off {
        return None;
    }
    let nth = FIRED.fetch_add(1, Ordering::Relaxed);
    let how = how.resolve(nth);
    let line = done_line(nth);
    // Stride 2 against 3 accents: every tint before any repeat.
    let tint = u8::try_from(nth.wrapping_mul(2) % 3).unwrap_or(0);
    let mut slot = host.0;
    slot.set(Some(Playing {
        how,
        line,
        tint,
        id: nth,
    }));
    // The timer belongs to the host, not here. Spawning it from a tap
    // handler tied it to whichever card was tapped, and a re-render or a
    // reorder of the list could drop the task mid-animation.
    (!how.shows_line()).then_some(line)
}

/// Draws whatever is in the slot.
#[component]
pub fn CelebrationHostView() -> Element {
    let host = use_context::<CelebrationHost>();
    let mut slot = host.0;

    // Clears the slot once the animation has had its time. Keyed on the
    // crossing's id, so a new celebration restarts the clock and a mere
    // re-render does not. This component lives at the app root and never
    // unmounts, so the timer cannot be cancelled out from under it.
    let playing = slot.read().to_owned();
    let current = playing.as_ref().map(|p| (p.id, p.how));
    use_effect(use_reactive(&current, move |current| {
        if let Some((id, how)) = current {
            spawn(async move {
                tokio::time::sleep(linger(how)).await;
                // Only clear this one. A later crossing has already
                // replaced it and owns the slot now.
                if slot.peek().as_ref().is_some_and(|p| p.id == id) {
                    slot.set(None);
                }
            });
        }
    }));

    let Some(Playing {
        how,
        line,
        tint,
        id,
    }) = playing
    else {
        return rsx! {};
    };
    match how {
        // Resolved before it reaches the slot, so neither can appear here.
        Celebration::Off | Celebration::Random => rsx! {},
        Celebration::Typewriter => rsx! {
            div { key: "{id}", class: "celebrate-text tint-{tint}", aria_hidden: "true",
                // The cursor is a sibling, not a border: inside the clipping
                // span it would eat into the animated width and swallow the
                // last character.
                span { class: "type-line",
                    span {
                        class: "type-text",
                        style: "--chars: {line.chars().count()};",
                        "{line}"
                    }
                    span { class: "type-cursor" }
                }
            }
        },
        Celebration::Stamp => rsx! {
            div { key: "{id}", class: "celebrate-text tint-{tint}", aria_hidden: "true",
                span { class: "stamp-line", "{line}" }
            }
        },
        Celebration::Scanline => rsx! {
            div { key: "{id}", class: "celebrate-scanline tint-{tint}", aria_hidden: "true" }
        },
        Celebration::Pulse => rsx! {
            div { key: "{id}", class: "celebrate-pulse tint-{tint}", aria_hidden: "true" }
        },
        Celebration::Sheen => rsx! {
            div { key: "{id}", class: "celebrate-sheen", aria_hidden: "true" }
        },
        Celebration::Poppers => rsx! {
            div { key: "{id}", class: "celebrate-poppers", aria_hidden: "true",
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
            div { key: "{id}", class: "celebrate-confetti", aria_hidden: "true",
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
